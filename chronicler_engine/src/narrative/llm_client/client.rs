//! [DOC: docs/system/llm_processing.md]

use crate::error::{EngineError, LlmFailure};

pub fn call_chat_completions(
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    system_prompt: &str,
    user_text: &str,
    title: Option<&str>,
    max_tokens: Option<u32>,
) -> crate::error::Result<super::request::ChatCompletionResult> {
    use super::request::{
        build_request_payload, configure_request, next_request_id, DEFAULT_MAX_TOKENS,
    };
    use super::response::handle_response;

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

    // Build request payload (pure function)
    let (payload, raw_request_json) =
        build_request_payload(model, system_prompt, user_text, max_tokens);
    // Configure HTTP request (pure function)
    let url = format!("{base_url}/chat/completions");
    let request = configure_request(&client, &url, &payload, api_key, title);
    tracing::info!("[LLM][req:{req_id}] Sending request to {url}");
    let res = request.send();
    // Handle response (delegates to extracted function)
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

pub fn call_openrouter_with_model(
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
    model: &str,
    max_tokens: Option<u32>,
) -> crate::error::Result<super::request::ChatCompletionResult> {
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

pub fn call_ollama(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    user_text: &str,
    max_tokens: Option<u32>,
) -> crate::error::Result<super::request::ChatCompletionResult> {
    call_chat_completions(
        base_url,
        None,
        model,
        system_prompt,
        user_text,
        None,
        max_tokens,
    )
}
