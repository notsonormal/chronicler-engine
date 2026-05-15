//! [DOC: docs/system/llm_processing.md]

use std::sync::atomic::{AtomicU64, Ordering};

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;

use crate::error::{EngineError, LlmFailure};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_request_id() -> u64 {
    REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst)
}

const DEFAULT_MAX_TOKENS: u32 = 2048;

#[derive(Debug)]
pub struct ChatCompletionResult {
    pub text: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
}

// [DOC: docs/system/llm_processing.md]
// Model selection is now connection-driven; these helpers are retained
// for backward compatibility during transition but delegate to Connection.

pub(crate) fn extract_content_from_response(
    json: &serde_json::Value,
) -> Option<(String, &'static str)> {
    let message = json.get("choices")?.get(0)?.get("message")?;

    // 1. Try content field
    if let Some(c) = message.get("content").and_then(|c| c.as_str()) {
        return Some((c.to_string(), "content"));
    }

    // 2. Try reasoning field
    if let Some(r) = message.get("reasoning").and_then(|r| r.as_str()) {
        return Some((r.to_string(), "reasoning"));
    }

    // 3. Try reasoning_content field (OpenRouter extended field)
    if let Some(rc) = message.get("reasoning_content").and_then(|rc| rc.as_str()) {
        return Some((rc.to_string(), "reasoning_content"));
    }

    None
}

/// Appends the Gemma 4 thinking-channel closure marker to bypass infinite thought loops.
// [DOC: docs/system/llm_processing.md section 8]
pub(crate) fn apply_gemma4_thinking_suffix(user_text: &str, model: &str) -> String {
    let m = model.to_lowercase();
    if m.contains("gemma-4") || m.contains("gemma4") {
        format!("{user_text}\n<|turn>model\n<|channel>thought\n<channel|>")
    } else {
        user_text.to_string()
    }
}

/// Strip leaked thinking/reasoning artifacts from LLM output.
/// Applies to all models as a defensive safety net.
#[allow(clippy::expect_used)]
pub(crate) fn sanitize_llm_output(text: &str) -> String {
    static RE_LEADING_CHANNEL: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*<channel\|>").expect("valid regex"));
    static RE_THOUGHT_BLOCK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?s)<thought>.*?</thought>").expect("valid regex"));
    static RE_CHANNEL_THOUGHT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?s)<\|channel>thought.*?<channel\|>").expect("valid regex"));
    static RE_TURN_MARKERS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"<\|turn>model|<turn\|>|<\|turn>").expect("valid regex"));

    let result = RE_LEADING_CHANNEL.replace(text, "");
    let result = RE_THOUGHT_BLOCK.replace_all(&result, "");
    let result = RE_CHANNEL_THOUGHT.replace_all(&result, "");
    let result = RE_TURN_MARKERS.replace_all(&result, "");

    // Normalize paragraph indentation.
    // [DOC: docs/system/llm_processing.md section 9]
    result
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Parse a raw HTTP response body from an LLM chat completions endpoint.
pub(crate) fn parse_chat_response(raw_response: &str, req_id: u64) -> crate::error::Result<String> {
    match serde_json::from_str::<serde_json::Value>(raw_response.trim_start()) {
        Ok(json_response) => {
            log::debug!("[LLM][req:{req_id}] Response JSON: {json_response:#}");

            if let Some(error) = json_response.get("error") {
                let error_msg = error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown API error");
                log::error!("[LLM][req:{req_id}] API error: {error_msg}");
                return Err(EngineError::Llm(LlmFailure::Http {
                    status: 200,
                    body: error_msg.to_string(),
                }));
            }

            if let Some((content, source)) = extract_content_from_response(&json_response) {
                let sanitized = sanitize_llm_output(&content);
                log::info!(
                    "[LLM][req:{req_id}] Extracted content via: {source} ({} chars)",
                    sanitized.len()
                );
                return Ok(sanitized);
            }

            // If we got here, the response structure was unexpected
            log::error!(
                "[LLM][req:{req_id}] Parse error: Could not find content in response structure"
            );
            log::error!(
                "[LLM][req:{req_id}] Response had keys: {:?}",
                json_response
                    .as_object()
                    .map(|m| m.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default()
            );
            Err(EngineError::Llm(LlmFailure::ParseError {
                raw_response: raw_response.to_string(),
                expected_format: "content or reasoning",
            }))
        }
        Err(e) => {
            log::error!("[LLM][req:{req_id}] JSON parse error: {e}");
            log::error!(
                "[LLM][req:{req_id}] Raw response that failed to parse: {}",
                raw_response.trim_start()
            );
            Err(EngineError::Llm(LlmFailure::ParseError {
                raw_response: raw_response.to_string(),
                expected_format: "valid JSON",
            }))
        }
    }
}

/// [DOC: docs/system/llm_processing.md]
pub fn call_chat_completions(
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    system_prompt: &str,
    user_text: &str,
    title: Option<&str>,
    max_tokens: Option<u32>,
) -> crate::error::Result<ChatCompletionResult> {
    let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
    let req_id = next_request_id();
    let start_time = std::time::Instant::now();

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| {
            EngineError::Llm(LlmFailure::Network {
                url: format!("{base_url}/chat/completions"),
                detail: format!("Failed to create HTTP client: {e}"),
            })
        })?;

    log::info!("[LLM][req:{req_id}] Using model: {model}");
    log::debug!(
        "[LLM][req:{req_id}] System prompt length: {} chars",
        system_prompt.len()
    );
    log::debug!(
        "[LLM][req:{req_id}] User text length: {} chars",
        user_text.len()
    );
    log::debug!("[LLM][req:{req_id}] Max tokens: {max_tokens}");

    let mut messages = vec![];
    if !system_prompt.is_empty() {
        messages.push(json!({
            "role": "system",
            "content": system_prompt
        }));
    }
    messages.push(json!({
        "role": "user",
        "content": user_text
    }));

    let payload = json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "max_tokens": max_tokens
    });

    let raw_request_json =
        serde_json::to_string(&payload).unwrap_or_else(|_| format!("{{\"model\":\"{model}\"}}"));

    log::debug!("[LLM][req:{req_id}] Request payload: {payload:#}");

    let url = format!("{base_url}/chat/completions");
    let mut request = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .json(&payload);

    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {key}"));
    }
    if let Some(t) = title {
        request = request.header("X-Title", t);
        request = request.header("HTTP-Referer", "https://github.com/chronicler-engine");
    }

    log::info!("[LLM][req:{req_id}] Sending request to {url}");
    let res = request.send();
    let header_time = start_time.elapsed();

    match res {
        Ok(response) => {
            let status = response.status();
            log::info!(
                "[LLM][req:{req_id}] Response status: {status} (headers after {:.2}s)",
                header_time.as_secs_f64()
            );

            // Log response headers for debugging
            log::debug!(
                "[LLM][req:{req_id}] Response headers: {:?}",
                response.headers()
            );

            if !status.is_success() {
                // Include the response body so the error message is actionable
                let error_body = response.text().unwrap_or_default();
                log::error!(
                    "[LLM][req:{req_id}] Non-success HTTP status: {status}. Body: {error_body}"
                );
                let snippet = if error_body.len() > 500 {
                    format!("{}...", &error_body[..500])
                } else {
                    error_body.clone()
                };
                return Err(EngineError::Llm(LlmFailure::Http {
                    status: status.as_u16(),
                    body: snippet,
                }));
            }

            // Try to parse JSON response - get raw text first to log on failure
            let raw_response = response.text().map_err(|e| {
                let elapsed = start_time.elapsed();
                log::error!(
                    "[LLM][req:{req_id}] Failed to read response body after {:.2}s: {e}",
                    elapsed.as_secs_f64()
                );
                log::error!(
                    "[LLM][req:{req_id}] This usually means: 1) Overall timeout (body still streaming), 2) Truncated gzip stream, 3) Server closed connection"
                );
                EngineError::Llm(LlmFailure::Network {
                    url: url.clone(),
                    detail: format!("Failed to read response body: {e}"),
                })
            })?;

            let body_time = start_time.elapsed();
            log::debug!(
                "[LLM][req:{req_id}] Raw response length: {} bytes (body after {:.2}s)",
                raw_response.len(),
                body_time.as_secs_f64()
            );

            let result = parse_chat_response(&raw_response, req_id);
            let total_time = start_time.elapsed();
            match &result {
                Ok(content) => {
                    log::info!(
                        "[LLM][req:{req_id}] Success: {} chars in {:.2}s total",
                        content.len(),
                        total_time.as_secs_f64()
                    );
                }
                Err(e) => {
                    log::error!(
                        "[LLM][req:{req_id}] Failed after {:.2}s: {e}",
                        total_time.as_secs_f64()
                    );
                }
            }
            let text = result?;
            Ok(ChatCompletionResult {
                text,
                system_prompt: system_prompt.to_string(),
                user_prompt: user_text.to_string(),
                raw_request_json,
                raw_response_json: raw_response,
            })
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            log::error!(
                "[LLM][req:{req_id}] Request failed after {:.2}s: {e}",
                elapsed.as_secs_f64()
            );
            Err(EngineError::Llm(LlmFailure::Network {
                url,
                detail: format!("Request failed: {e}"),
            }))
        }
    }
}

/// [DOC: docs/system/llm_processing.md]
pub fn call_openrouter_with_model(
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
    model: &str,
    max_tokens: Option<u32>,
) -> crate::error::Result<ChatCompletionResult> {
    call_chat_completions(
        "https://openrouter.ai/api/v1",
        Some(api_key),
        model,
        system_prompt,
        user_text,
        Some("Chronicler Engine"),
        max_tokens,
    )
}

/// [DOC: docs/system/llm_processing.md]
pub fn call_ollama(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    user_text: &str,
    max_tokens: Option<u32>,
) -> crate::error::Result<ChatCompletionResult> {
    let user_text = apply_gemma4_thinking_suffix(user_text, model);
    call_chat_completions(
        base_url,
        None,
        model,
        system_prompt,
        &user_text,
        None,
        max_tokens,
    )
}
