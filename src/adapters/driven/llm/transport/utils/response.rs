//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! LLM response parsing

use crate::error::{EngineError, LlmFailure};

pub fn extract_content_from_response(json: &serde_json::Value) -> Option<(String, &'static str)> {
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

pub fn parse_chat_response(raw_response: &str, req_id: u64) -> crate::error::Result<String> {
    match serde_json::from_str::<serde_json::Value>(raw_response.trim_start()) {
        Ok(json_response) => {
            tracing::debug!("[LLM][req:{req_id}] Response JSON: {json_response:#}");

            if let Some(error) = json_response.get("error") {
                let error_msg = error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown API error");
                tracing::error!("[LLM][req:{req_id}] API error: {error_msg}");
                return Err(EngineError::Llm(LlmFailure::Http {
                    status: 200,
                    body: error_msg.to_string(),
                }));
            }

            if let Some((content, source)) = extract_content_from_response(&json_response) {
                tracing::info!(
                    "[LLM][req:{req_id}] Extracted content via: {source} ({} chars)",
                    content.len()
                );
                return Ok(content);
            }

            // If we got here, the response structure was unexpected
            tracing::error!(
                "[LLM][req:{req_id}] Parse error: Could not find content in response structure"
            );
            tracing::error!(
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
            tracing::error!("[LLM][req:{req_id}] JSON parse error: {e}");
            tracing::error!(
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

pub fn handle_response(
    response: reqwest::blocking::Response,
    req_id: u64,
    start_time: std::time::Instant,
    url: &str,
    system_prompt: &str,
    user_text: &str,
    raw_request_json: String,
) -> crate::error::Result<super::request::ChatCompletionResult> {
    let status = response.status();
    let header_time = start_time.elapsed();
    tracing::info!(
        "[LLM][req:{req_id}] Response status: {status} (headers after {:.2}s)",
        header_time.as_secs_f64()
    );
    // Log response headers for debugging
    tracing::debug!(
        "[LLM][req:{req_id}] Response headers: {:?}",
        response.headers()
    );
    if !status.is_success() {
        // Include the response body so the error message is actionable
        let error_body = response.text().unwrap_or_default();
        tracing::error!(
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
        tracing::error!(
            "[LLM][req:{req_id}] Failed to read response body after {:.2}s: {e}",
            elapsed.as_secs_f64()
        );
        tracing::error!(
            "[LLM][req:{req_id}] This usually means: 1) Overall timeout (body still streaming), 2) Truncated gzip stream, 3) Server closed connection"
        );
        EngineError::Llm(LlmFailure::Network {
            url: url.to_string(),
            detail: format!("Failed to read response body: {e}"),
        })
    })?;
    let body_time = start_time.elapsed();
    tracing::debug!(
        "[LLM][req:{req_id}] Raw response length: {} bytes (body after {:.2}s)",
        raw_response.len(),
        body_time.as_secs_f64()
    );
    let result = parse_chat_response(&raw_response, req_id);
    let total_time = start_time.elapsed();
    match &result {
        Ok(content) => {
            tracing::info!(
                "[LLM][req:{req_id}] Success: {} chars in {:.2}s total",
                content.len(),
                total_time.as_secs_f64()
            );
        }
        Err(e) => {
            tracing::error!(
                "[LLM][req:{req_id}] Failed after {:.2}s: {e}",
                total_time.as_secs_f64()
            );
        }
    }
    let text = result?;
    Ok(super::request::ChatCompletionResult {
        text,
        system_prompt: system_prompt.to_string(),
        user_prompt: user_text.to_string(),
        raw_request_json,
        raw_response_json: raw_response,
    })
}
