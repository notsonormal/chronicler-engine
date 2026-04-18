//! OpenRouter HTTP client
//!
//! This module handles the actual HTTP communication with OpenRouter API.
//! It's isolated to allow easy exclusion from coverage (requires external API).
//!
//! Two model configurations are supported:
//! - `LLM_MODEL`: Primary model for narrative generation (default: `z-ai/glm-4.5-air:free`)
//! - `QUANTIFIER_MODEL`: Secondary model for scene quantification (default: `z-ai/glm-4.5-air:free`)

use serde_json::json;

/// Get the configured LLM model for narrative generation.
///
/// Reads the `LLM_MODEL` environment variable, defaulting to `z-ai/glm-4.5-air:free`.
pub fn get_llm_model() -> String {
    std::env::var("LLM_MODEL").unwrap_or_else(|_| "z-ai/glm-4.5-air:free".to_string())
}

/// Get the configured LLM model for scene quantification.
///
/// Reads the `QUANTIFIER_MODEL` environment variable, defaulting to `z-ai/glm-4.5-air:free`.
/// The quantifier uses a separate model configuration so it can run on a
/// different (typically cheaper/faster) model than the main narrative generator.
pub fn get_quantifier_model() -> String {
    std::env::var("QUANTIFIER_MODEL").unwrap_or_else(|_| "z-ai/glm-4.5-air:free".to_string())
}

/// Call OpenRouter API with a specific model and return the response content.
///
/// This is the core implementation that all OpenRouter calls route through.
/// The `model` parameter specifies which LLM to use (e.g., `z-ai/glm-4.5-air:free`).
pub fn call_openrouter_with_model(
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
    model: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

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

                    // Helper to check if Option<&str> is both non-null and non-empty
                    fn is_non_empty(s: Option<&str>) -> bool {
                        s.map(|s| !s.is_empty()).unwrap_or(false)
                    }

                    // Extract content with robust fallback chain
                    // Some models return empty string "", some return null - need to handle both
                    let content_source: &str;
                    let content: Option<String> = {
                        // 1. Try content field (only if non-null AND non-empty)
                        let c = json_response["choices"]
                            .get(0)
                            .and_then(|c| c.get("message"))
                            .and_then(|m| m.get("content"))
                            .and_then(|c| c.as_str());

                        if is_non_empty(c) {
                            content_source = "content";
                            Some(c.unwrap().to_string())
                        } else {
                            // 2. Try reasoning field (only if non-null AND non-empty)
                            let r = json_response["choices"]
                                .get(0)
                                .and_then(|c| c.get("message"))
                                .and_then(|m| m.get("reasoning"))
                                .and_then(|r| r.as_str());

                            if is_non_empty(r) {
                                content_source = "reasoning";
                                Some(r.unwrap().to_string())
                            } else {
                                // 3. Try reasoning_content field (OpenRouter extended field)
                                let rc = json_response["choices"]
                                    .get(0)
                                    .and_then(|c| c.get("message"))
                                    .and_then(|m| m.get("reasoning_content"))
                                    .and_then(|rc| rc.as_str());

                                if is_non_empty(rc) {
                                    content_source = "reasoning_content";
                                    Some(rc.unwrap().to_string())
                                } else {
                                    content_source = "none";
                                    None
                                }
                            }
                        }
                    };

                    if let Some(content) = content {
                        log::info!(
                            "[LLM] Extracted content via: {} ({} chars)",
                            content_source,
                            content.len()
                        );
                        return Ok(content);
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

/// Call OpenRouter API using the primary narrative model.
///
/// Convenience wrapper that reads `LLM_MODEL` from the environment
/// and delegates to [`call_openrouter_with_model`].
pub fn call_openrouter(
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    let model = get_llm_model();
    call_openrouter_with_model(api_key, system_prompt, user_text, &model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_llm_model_default() {
        // When LLM_MODEL is not set, should return default
        let model = get_llm_model();
        // The default may vary if env var is set in CI, so just check it's non-empty
        assert!(!model.is_empty());
    }

    #[test]
    fn test_get_quantifier_model_default() {
        // When QUANTIFIER_MODEL is not set, should return default
        let model = get_quantifier_model();
        assert!(!model.is_empty());
    }

    #[test]
    fn test_get_llm_model_returns_default_value() {
        // Verify the expected default model string
        // Note: This test may fail if LLM_MODEL is set in the environment
        let default_model = std::env::var("LLM_MODEL");
        if default_model.is_err() {
            assert_eq!(get_llm_model(), "z-ai/glm-4.5-air:free");
        }
    }

    #[test]
    fn test_get_quantifier_model_returns_default_value() {
        // Verify the expected default model string
        // Note: This test may fail if QUANTIFIER_MODEL is set in the environment
        let default_model = std::env::var("QUANTIFIER_MODEL");
        if default_model.is_err() {
            assert_eq!(get_quantifier_model(), "z-ai/glm-4.5-air:free");
        }
    }
}
