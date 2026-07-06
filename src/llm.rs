use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde_json::Value;

fn build_client() -> reqwest::blocking::Client {
    let timeout_secs = std::env::var("LLM_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(120);
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .expect("Failed to build HTTP client")
}

pub enum Provider {
    OpenAI,
    Anthropic,
    Groq,
    Gemini,
}

fn detect_provider(api_key: &str) -> Provider {
    match std::env::var("LLM_PROVIDER").as_deref() {
        Ok("anthropic") => return Provider::Anthropic,
        Ok("groq") => return Provider::Groq,
        Ok("gemini") => return Provider::Gemini,
        _ => {}
    }
    if api_key.starts_with("sk-ant-") {
        Provider::Anthropic
    } else if api_key.starts_with("gsk_") {
        Provider::Groq
    } else if api_key.starts_with("AIza") {
        Provider::Gemini
    } else {
        Provider::OpenAI
    }
}

/// Send a prompt to the configured LLM and return the generated text.
pub fn generate(prompt: &str, api_key: &str) -> Result<String> {
    match detect_provider(api_key) {
        Provider::OpenAI => openai_generate(prompt, api_key),
        Provider::Anthropic => anthropic_generate(prompt, api_key),
        Provider::Groq => groq_generate(prompt, api_key),
        Provider::Gemini => gemini_generate(prompt, api_key),
    }
}

fn openai_compatible_request(
    prompt: &str,
    api_key: &str,
    base_url: &str,
    default_model: &str,
) -> Result<String> {
    // Allow override via LLM_BASE_URL env var
    let url = std::env::var("LLM_BASE_URL").unwrap_or_else(|_| base_url.to_string());
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| default_model.into());

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.3,
    });

    let client = build_client();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .with_context(|| format!("API request failed (timeout: {}s)", {
            std::env::var("LLM_TIMEOUT").unwrap_or_else(|_| "120".into())
        }))?;

    let status = resp.status();
    let body_text = resp.text().context("Failed to read response body")?;
    let json: Value = serde_json::from_str(&body_text)
        .with_context(|| format!("Failed to parse response as JSON: {body_text}"))?;

    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or(&body_text);
        bail!("API error ({status}): {msg} (tried model `{model}` — set LLM_MODEL to override)");
    }

    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .context("No content in API response")?;

    Ok(text.to_string())
}

fn gemini_generate(prompt: &str, api_key: &str) -> Result<String> {
    let default_model = "gemini-1.5-flash";
    // Try OpenAI-compatible endpoint with Bearer auth
    let result = openai_compatible_request(
        prompt,
        api_key,
        "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
        default_model,
    );
    if result.is_ok() {
        return result;
    }
    // Some Gemini endpoints need ?key= query param — retry with native endpoint
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| default_model.into());
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );
    let body = serde_json::json!({
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {"temperature": 0.3},
    });
    let client = build_client();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .with_context(|| format!("Gemini native API request failed (timeout: {}s)", {
            std::env::var("LLM_TIMEOUT").unwrap_or_else(|_| "120".into())
        }))?;
    let status = resp.status();
    let body_text = resp.text().context("Failed to read Gemini response")?;
    let json: Value = serde_json::from_str(&body_text)
        .with_context(|| format!("Failed to parse Gemini response: {body_text}"))?;
    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or(&body_text);
        bail!("Gemini API error ({status}): {msg} (tried model `{model}` — set LLM_MODEL to override)");
    }
    let text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("No text in Gemini response")?;
    Ok(text.to_string())
}

fn groq_generate(prompt: &str, api_key: &str) -> Result<String> {
    openai_compatible_request(
        prompt,
        api_key,
        "https://api.groq.com/openai/v1/chat/completions",
        "llama-3.3-70b-versatile",
    )
}

fn openai_generate(prompt: &str, api_key: &str) -> Result<String> {
    openai_compatible_request(
        prompt,
        api_key,
        "https://api.openai.com/v1/chat/completions",
        "gpt-4o",
    )
}

fn anthropic_generate(prompt: &str, api_key: &str) -> Result<String> {
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "claude-sonnet-4-20250514".into());

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{"role": "user", "content": prompt}],
    });

    let client = build_client();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .with_context(|| format!("Anthropic API request failed (timeout: {}s)", {
            std::env::var("LLM_TIMEOUT").unwrap_or_else(|_| "120".into())
        }))?;

    let status = resp.status();
    let json: Value = resp.json().context("Failed to parse Anthropic response")?;

    if !status.is_success() {
        let msg = json["error"]["message"].as_str().unwrap_or("unknown");
        bail!("Anthropic API error ({status}): {msg}");
    }

    let text = json["content"][0]["text"]
        .as_str()
        .context("No content in Anthropic response")?;

    Ok(text.to_string())
}
