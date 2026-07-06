use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tiktoken_rs::cl100k_base;
use tree_sitter::{Parser, QueryCursor};

use super::queries::{class_query, decorated_query, function_query, import_from_query, import_query};
use super::skeleton::{ClassDef, FileSkeleton, FnDef, Import};

/// Parse a Python file and return a token-stripped skeleton.
pub fn extract_skeleton(path: &Path) -> Result<FileSkeleton> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::language())
        .context("Failed to set Python language")?;
    let tree = parser.parse(&source, None).context("Failed to parse source")?;
    let root = tree.root_node();

    // --- Extract imports ---
    let mut imports = Vec::new();

    // `import os` → module="os", name=None
    let query = import_query();
    let mut cursor = QueryCursor::new();
    for match_ in cursor.matches(&query, root, source.as_bytes()) {
        let mut module = None;
        for cap in match_.captures {
            let name = query.capture_names()[cap.index as usize];
            if name == "import_name" {
                module = Some(cap.node.utf8_text(source.as_bytes())?.to_string());
            }
        }
        if let Some(mod_name) = module {
            imports.push(Import {
                name: None,
                module: Some(mod_name),
            });
        }
    }

    // `from fastapi import APIRouter` → module="fastapi", name="APIRouter"
    let query = import_from_query();
    let mut cursor = QueryCursor::new();
    for match_ in cursor.matches(&query, root, source.as_bytes()) {
        let mut from_module = None;
        let mut import_name = None;
        for cap in match_.captures {
            let name = query.capture_names()[cap.index as usize];
            match name {
                "from_module" => from_module = Some(cap.node.utf8_text(source.as_bytes())?.to_string()),
                "import_name" => import_name = Some(cap.node.utf8_text(source.as_bytes())?.to_string()),
                _ => {}
            }
        }
        imports.push(Import {
            name: import_name,
            module: from_module,
        });
    }

    // --- Extract function definitions ---
    let mut functions: Vec<FnDef> = Vec::new();
    let mut body_ranges: Vec<std::ops::Range<usize>> = Vec::new();

    // Decorated functions (FastAPI routes)
    let query = decorated_query();
    let mut cursor = QueryCursor::new();
    for match_ in cursor.matches(&query, root, source.as_bytes()) {
        let mut func_name = None;
        let mut decorator = None;
        let mut params = None;
        for cap in match_.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            match cap_name {
                "func_name" => {
                    func_name = Some(cap.node.utf8_text(source.as_bytes())?.to_string());
                }
                "decorator" => {
                    decorator = Some(cap.node.utf8_text(source.as_bytes())?.to_string());
                }
                "func_params" => {
                    params = Some(cap.node.utf8_text(source.as_bytes())?.to_string());
                }
                "func_body" => {
                    body_ranges.push(cap.node.byte_range());
                }
                _ => {}
            }
        }
        if let Some(name) = func_name {
            let decorator_text = decorator.unwrap_or_default();
            let param_text = params.unwrap_or_default();
            let sig = format!("{decorator_text}\ndef {name}{param_text}:");
            functions.push(FnDef {
                name,
                is_decorated: true,
                signature: sig,
            });
        }
    }

    // Bare top-level functions
    let query = function_query();
    let mut cursor = QueryCursor::new();
    for match_ in cursor.matches(&query, root, source.as_bytes()) {
        let mut func_name = None;
        let mut params = None;
        for cap in match_.captures {
            let cap_name = query.capture_names()[cap.index as usize];
            match cap_name {
                "func_name" => {
                    func_name = Some(cap.node.utf8_text(source.as_bytes())?.to_string());
                }
                "func_params" => {
                    params = Some(cap.node.utf8_text(source.as_bytes())?.to_string());
                }
                "func_body" => {
                    body_ranges.push(cap.node.byte_range());
                }
                _ => {}
            }
        }
        if let Some(name) = func_name {
            let param_text = params.unwrap_or_default();
            let sig = format!("def {name}{param_text}:");
            functions.push(FnDef {
                name,
                is_decorated: false,
                signature: sig,
            });
        }
    }

    // --- Extract class definitions ---
    let mut classes: Vec<ClassDef> = Vec::new();
    let query = class_query();
    let mut cursor = QueryCursor::new();
    for match_ in cursor.matches(&query, root, source.as_bytes()) {
        for cap in match_.captures {
            let name = query.capture_names()[cap.index as usize];
            if name == "class_name" {
                classes.push(ClassDef {
                    name: cap.node.utf8_text(source.as_bytes())?.to_string(),
                });
            }
        }
    }

    // Sort and deduplicate body ranges
    body_ranges.sort_by_key(|r| r.start);
    body_ranges.dedup_by_key(|r| (r.start, r.end));

    let stripped = strip_bodies(&source, &body_ranges);
    let bpe = cl100k_base().context("Failed to load tiktoken tokenizer")?;
    let token_count = bpe.encode_with_special_tokens(&stripped).len() as u32;

    Ok(FileSkeleton {
        path: path.to_string_lossy().to_string(),
        imports,
        functions,
        classes,
        source_text: stripped,
        raw_source: source,
        token_count,
    })
}

/// Keep key lines (return, raise, await, async with, yield, and assignments from await).
fn filter_body(source: &str, range: &std::ops::Range<usize>) -> String {
    let body = &source[range.start..range.end];
    let mut out = String::new();
    let mut kept = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("return")
            || trimmed.starts_with("raise")
            || trimmed.starts_with("await")
            || trimmed.starts_with("async with")
            || trimmed.starts_with("yield")
            || trimmed.starts_with("response")
            || trimmed.starts_with("state.")
            || trimmed.contains(" = await ")
            || trimmed.contains(" = response.")
        {
            out.push_str(line);
            out.push('\n');
            kept = true;
        }
    }
    if kept {
        out
    } else {
        "    ...\n".to_string()
    }
}

/// Replace every byte range in `ranges` (sorted, non-overlapping) with key lines or `...`.
fn strip_bodies(source: &str, ranges: &[std::ops::Range<usize>]) -> String {
    let mut result = String::with_capacity(source.len());
    let mut last_end = 0usize;

    for range in ranges {
        // Append text before this body
        result.push_str(&source[last_end..range.start]);
        // Append key lines from the body
        result.push_str(&filter_body(source, range));
        last_end = range.end;
    }
    // Append remaining text after the last body
    result.push_str(&source[last_end..]);

    result
}
