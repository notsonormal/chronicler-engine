//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM response parsing

use crate::error::{EngineError, LlmFailure};

fn missing_message_error(
    json_response: &serde_json::Value,
    raw_response: &str,
    req_id: u64,
) -> EngineError {
    tracing::error!(
        "[LLM][req:{req_id}] Parse error: Could not find choices[0].message in response structure"
    );
    tracing::error!(
        "[LLM][req:{req_id}] Response had keys: {:?}",
        json_response
            .as_object()
            .map(|m| m.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    );
    EngineError::Llm(LlmFailure::ParseError {
        raw_response: raw_response.to_string(),
        expected_format: "choices[0].message.content",
    })
}

fn token_counts(json_response: &serde_json::Value) -> (Option<u64>, Option<u64>) {
    let usage = json_response.get("usage");
    let completion_tokens = usage
        .and_then(|usage| usage.get("completion_tokens"))
        .and_then(serde_json::Value::as_u64);
    let reasoning_tokens = usage
        .and_then(|usage| usage.get("completion_tokens_details"))
        .and_then(|details| details.get("reasoning_tokens"))
        .and_then(serde_json::Value::as_u64);
    (completion_tokens, reasoning_tokens)
}

/// The one place the "what counts as an answer" rule lives, shared by every provider
/// that parses a chat-completions body.
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

            let Some(choice) = json_response.get("choices").and_then(|c| c.get(0)) else {
                return Err(missing_message_error(&json_response, raw_response, req_id));
            };
            let Some(message) = choice.get("message").filter(|message| message.is_object()) else {
                return Err(missing_message_error(&json_response, raw_response, req_id));
            };

            // `reasoning` and `reasoning_content` are separate channels: only
            // `choices[0].message.content` can become the answer.
            if let Some(content) = message
                .get("content")
                .and_then(|c| c.as_str())
                .filter(|c| !c.trim().is_empty())
            {
                tracing::info!(
                    "[LLM][req:{req_id}] Extracted content ({} chars)",
                    content.len()
                );
                return Ok(content.to_string());
            }

            // `finish_reason: "length"` names the cause of the missing answer: the
            // model spent its whole output budget before writing one.
            if choice.get("finish_reason").and_then(|f| f.as_str()) == Some("length") {
                let (completion_tokens, reasoning_tokens) = token_counts(&json_response);
                tracing::error!(
                    "[LLM][req:{req_id}] Token budget spent with no answer (completion_tokens: {completion_tokens:?}, reasoning_tokens: {reasoning_tokens:?})"
                );
                return Err(EngineError::Llm(LlmFailure::TokenBudgetSpent {
                    raw_response: raw_response.to_string(),
                    completion_tokens,
                    reasoning_tokens,
                }));
            }

            tracing::error!(
                "[LLM][req:{req_id}] Empty response: choices[0].message carries no answer"
            );
            Err(EngineError::Llm(LlmFailure::EmptyResponse {
                raw_response: raw_response.to_string(),
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
