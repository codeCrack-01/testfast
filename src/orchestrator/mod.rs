use anyhow::{Context, Result};
use tiktoken_rs::cl100k_base;

use crate::ast_engine::skeleton::FileSkeleton;
use crate::context_mgr::CoverageDelta;

/// The final assembled prompt ready to send to an LLM.
pub struct Prompt {
    pub text: String,
    pub token_count: u32,
}

/// Build a prompt from source context + uncovered functions.
/// `style_context` is optional test style guide (from test_agent.md).
/// When `more` is true, include the full stripped source for richer context.
pub fn build_prompt(
    source: &FileSkeleton,
    deltas: &CoverageDelta,
    style_context: &str,
    more: bool,
) -> Result<Prompt> {
    let mut text = String::new();

    text.push_str("You are a test generator for a FastAPI application.\n");
    text.push_str("Generate pytest tests for the uncovered functions listed below.\n\n");

    text.push_str(&format!("## Module: {}\n\n", source.path));

    if more {
        // Full source (unmodified) for maximum context
        text.push_str("### Full Source\n\n```python\n");
        text.push_str(&source.raw_source);
        text.push_str("\n```\n\n");
    } else {
        // Compact source — stripped bodies but keeps return/raise/await lines
        text.push_str("### Source\n\n```python\n");
        text.push_str(&source.source_text);
        text.push_str("\n```\n\n");
    }

    text.push_str("## Functions to Test (uncovered)\n\n");
    for func in &deltas.uncovered {
        let kind = if func.is_decorated { "route" } else { "helper" };
        text.push_str(&format!("- `{}` ({})\n", func.name, kind));
    }

    if !style_context.is_empty() {
        text.push_str("\n## Test Style Guide\n\n");
        text.push_str(style_context);
        text.push('\n');
    }

    // Derive the Python module name from the file path
    let module_name = std::path::Path::new(&source.path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("app");

    text.push_str("\n## Instructions\n\n");
    text.push_str("Output ONLY valid Python code — no markdown, no headings, no explanations, no notes. ");
    text.push_str("Put any commentary inside Python comments (# ...).\n\n");
    text.push_str("Rules:\n");
    text.push_str(&format!("- Import the real app: `from {} import app` (or the variable used in decorators above)\n", module_name));
    text.push_str("- Use `TestClient` for route endpoints\n");
    text.push_str("- Write one `test_<name>` function per uncovered function listed above\n");
    text.push_str("- Use pytest assertions (`assert`, `pytest.raises`, etc.)\n");
    text.push_str("- For async test functions, use `@pytest.mark.anyio` (NOT @pytest.mark.asyncio)\n");
    text.push_str("- If the code reads env vars at module level (`os.getenv` in imports), patch the module attribute after import with `monkeypatch.setattr(\"module.VAR_NAME\", \"value\")` — `setenv` is too late\n");
    text.push_str("- Use correct expected values from the source code above (check actual responses)\n");
    text.push_str("- Do NOT create a new FastAPI() app — import the existing one\n");
    text.push_str("- Only test the functions listed in 'Functions to Test', not FastAPI built-ins\n");
    text.push_str("- Do not explain the code — the test file should be runnable as-is\n");

    let bpe = cl100k_base().context("Failed to load tiktoken tokenizer")?;
    let token_count = bpe.encode_with_special_tokens(&text).len() as u32;

    Ok(Prompt { text, token_count })
}

/// Build a prompt to fix failing tests.
pub fn build_fix_prompt(
    source: &FileSkeleton,
    current_code: &str,
    pytest_output: &str,
    more: bool,
) -> Result<Prompt> {
    let mut text = String::new();

    text.push_str("The following tests failed when run against the FastAPI application.\n");
    text.push_str("Fix the failing tests and output ONLY the corrected Python code.\n\n");

    text.push_str(&format!("## Module: {}\n\n", source.path));

    if more {
        text.push_str("### Full Source\n\n```python\n");
        text.push_str(&source.raw_source);
        text.push_str("\n```\n\n");
    } else {
        text.push_str("### Source\n\n```python\n");
        text.push_str(&source.source_text);
        text.push_str("\n```\n\n");
    }

    text.push_str("## Current Test Code (with failures)\n\n```python\n");
    text.push_str(current_code);
    text.push_str("\n```\n\n");

    text.push_str("## Pytest Failure Output\n\n```\n");
    text.push_str(pytest_output);
    text.push_str("\n```\n\n");

    text.push_str("## Instructions\n\n");
    text.push_str("Output ONLY the corrected Python code — no markdown, no explanations.\n");
    text.push_str("Fix only the failing tests. Keep passing tests unchanged.\n");
    text.push_str("Use the source code above to determine the correct expected values.\n");
    text.push_str("For async tests use `@pytest.mark.anyio` (NOT @pytest.mark.asyncio).\n");

    let bpe = cl100k_base().context("Failed to load tiktoken tokenizer")?;
    let token_count = bpe.encode_with_special_tokens(&text).len() as u32;

    Ok(Prompt { text, token_count })
}
