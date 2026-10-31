//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM client implementation

use super::request::{ChatCompletionRequest, ChatCompletionsEndpoint};
use crate::error::{EngineError, LlmFailure};

pub fn call_chat_completions(
    endpoint: &ChatCompletionsEndpoint<'_>,
    request: &ChatCompletionRequest<'_>,
) -> crate::error::Result<super::request::ChatCompletionResult> {
    use super::request::{build_request_payload, configure_request, next_request_id};
    use super::response::handle_response;

    let ChatCompletionRequest {
        model,
        system_prompt,
        user_text,
        reasoning_effort,
        ..
    } = *request;
    let base_url = endpoint.base_url;
    let max_tokens = request.resolved_max_tokens();
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

    tracing::info!("[LLM][req:{req_id}] Using model: {model}");
    tracing::debug!(
        "[LLM][req:{req_id}] System prompt length: {} chars",
        system_prompt.len()
    );
    tracing::debug!(
        "[LLM][req:{req_id}] User text length: {} chars",
        user_text.len()
    );
    tracing::debug!("[LLM][req:{req_id}] Max tokens: {max_tokens}");
    tracing::debug!(
        "[LLM][req:{req_id}] Reasoning effort: {}",
        reasoning_effort.map_or("provider default", |effort| effort.as_str())
    );

    let (payload, raw_request_json) = build_request_payload(request);
    let url = format!("{base_url}/chat/completions");
    let http_request = configure_request(&client, &url, &payload, endpoint);
    tracing::info!("[LLM][req:{req_id}] Sending request to {url}");
    let res = http_request.send();
    match res {
        Ok(response) => handle_response(
            response,
            req_id,
            start_time,
            &url,
            system_prompt,
            user_text,
            raw_request_json,
        ),
        Err(e) => {
            let elapsed = start_time.elapsed();
            tracing::error!(
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
