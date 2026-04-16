//! OpenRouter HTTP client
//!
//! This module handles the actual HTTP communication with OpenRouter API.
//! It's isolated to allow easy exclusion from coverage (requires external API).

use serde_json::json;

/// Call OpenRouter API and return the response content
pub fn call_openrouter(
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "z-ai/glm-4.5-air:free".to_string());

    log::info!("[LLM] Using model: {model}");
    log::debug!("[LLM] System prompt length: {} chars", system_prompt.len());
    log::debug!("[LLM] User text length: {} chars", user_text.len());

    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_text
            }
        ],
        "stream": false
    });

    let res = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .header("HTTP-X-Title", "Chronicler Engine")
        .header("Accept-Encoding", "gzip, deflate")
        .json(&payload)
        .send();

    match res {
        Ok(response) => {
            let status = response.status();
            log::info!("[LLM] Response status: {status}");

            // Log response headers for debugging
            log::debug!("[LLM] Response headers: {:?}", response.headers());

            if !status.is_success() {
                log::error!("[LLM] Non-success HTTP status: {status}");
                return Err(format!("Error communicating with OpenRouter: {status}"));
            }

            // Try to parse JSON response - get raw text first to log on failure
            let raw_response = response.text().map_err(|e| {
                log::error!("[LLM] Failed to read response body: {e}");
                log::error!("[LLM] This usually means: 1) Network issue, 2) Invalid encoding, 3) Server closed connection");
                format!("Failed to read response body: {e}")
            })?;

            log::debug!("[LLM] Raw response length: {} bytes", raw_response.len());
            log::debug!(
                "[LLM] Raw response (first 500 chars): {}",
                &raw_response[..raw_response.len().min(500)]
            );

            match serde_json::from_str::<serde_json::Value>(&raw_response) {
                Ok(json_response) => {
                    log::debug!("[LLM] Raw JSON response: {json_response:?}");

                    // Check for API-level errors in response
                    if let Some(error) = json_response.get("error") {
                        let error_msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown API error");
                        log::error!("[LLM] API error: {error_msg}");
                        return Err(format!("LLM API error: {error_msg}"));
                    }

                    // Try to extract content from the standard response format
                    if let Some(content) = json_response["choices"]
                        .get(0)
                        .and_then(|c| c.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                    {
                        log::info!(
                            "[LLM] Successfully parsed content ({} chars)",
                            content.len()
                        );
                        return Ok(content.to_string());
                    }

                    // If we got here, the response structure was unexpected
                    log::error!("[LLM] Parse error: Could not find content in response structure");
                    log::error!(
                        "[LLM] Response had keys: {:?}",
                        json_response
                            .as_object()
                            .map(|m| m.keys().cloned().collect::<Vec<_>>())
                            .unwrap_or_default()
                    );
                    Err("The world seems to hold its breath (parse error).".to_string())
                }
                Err(e) => {
                    log::error!("[LLM] JSON parse error: {e}");
                    log::error!("[LLM] Raw response that failed to parse: {raw_response}");
                    Err(format!("Failed to parse LLM response: {e}"))
                }
            }
        }
        Err(e) => {
            log::error!("[LLM] Request failed: {e}");
            Err(format!("Request failed: {e}"))
        }
    }
}
