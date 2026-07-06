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
/// The LLM will be asked to generate pytest tests for the deltas.
pub fn build_prompt(source: &FileSkeleton, deltas: &CoverageDelta) -> Result<Prompt> {
    let mut text = String::new();

    text.push_str("You are a test generator for a FastAPI application.\n");
    text.push_str("Generate pytest tests for the uncovered functions listed below.\n\n");

    text.push_str("## Source Code\n\n");
    text.push_str("```python\n");
    text.push_str(&source.source_text);
    text.push_str("\n```\n\n");

    text.push_str("## Functions to Test (uncovered)\n\n");
    for func in &deltas.uncovered {
        let kind = if func.is_decorated { "route" } else { "helper" };
        text.push_str(&format!("- `{}` ({})\n", func.name, kind));
    }

    text.push_str("\n## Instructions\n\n");
    text.push_str("Write pytest functions for each uncovered function above. ");
    text.push_str("Use `TestClient` from FastAPI for route endpoints.\n");

    let bpe = cl100k_base().context("Failed to load tiktoken tokenizer")?;
    let token_count = bpe.encode_with_special_tokens(&text).len() as u32;

    Ok(Prompt { text, token_count })
}
