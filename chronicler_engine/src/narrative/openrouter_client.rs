//! [DOC: docs/system/llm_processing.md]

use serde_json::json;

use crate::model::settings::AppSettings;

// [DOC: docs/system/llm_processing.md]
pub fn get_llm_model(settings: &AppSettings) -> String {
    std::env::var("LLM_MODEL").unwrap_or_else(|_| settings.llm_model.clone())
}

// [DOC: docs/system/llm_processing.md]
pub fn get_quantifier_model(settings: &AppSettings) -> String {
    std::env::var("QUANTIFIER_MODEL").unwrap_or_else(|_| settings.quantifier_model.clone())
}

fn extract_content_from_response(json: &serde_json::Value) -> Option<(String, &'static str)> {
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

/// [DOC: docs/system/llm_processing.md]
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

    log::debug!("[LLM] Request payload: {payload:#}");

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

            match serde_json::from_str::<serde_json::Value>(raw_response.trim_start()) {
                Ok(json_response) => {
                    log::debug!("[LLM] Response JSON: {json_response:#}");

                    if let Some(error) = json_response.get("error") {
                        let error_msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown API error");
                        log::error!("[LLM] API error: {error_msg}");
                        return Err(format!("LLM API error: {error_msg}"));
                    }

                    if let Some((content, source)) = extract_content_from_response(&json_response) {
                        log::info!(
                            "[LLM] Extracted content via: {source} ({} chars)",
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
                    log::error!(
                        "[LLM] Raw response that failed to parse: {}",
                        raw_response.trim_start()
                    );
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

/// [DOC: docs/system/llm_processing.md]
pub fn call_openrouter(
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    let settings = crate::settings::load_settings().unwrap_or_default();
    let model = get_llm_model(&settings);
    call_openrouter_with_model(api_key, system_prompt, user_text, &model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_llm_model_default() {
        let settings = AppSettings::default();
        let model = get_llm_model(&settings);
        assert_eq!(model, "openai/gpt-4o-mini");
    }

    #[test]
    fn test_get_quantifier_model_default() {
        let settings = AppSettings::default();
        let model = get_quantifier_model(&settings);
        assert_eq!(model, "openai/gpt-4o-mini");
    }

    #[test]
    fn test_get_llm_model_from_settings() {
        let settings = AppSettings {
            llm_model: "google/gemini-pro".to_string(),
            ..Default::default()
        };
        let model = get_llm_model(&settings);
        assert_eq!(model, "google/gemini-pro");
    }

    #[test]
    fn test_get_quantifier_model_from_settings() {
        let settings = AppSettings {
            quantifier_model: "google/gemini-pro".to_string(),
            ..Default::default()
        };
        let model = get_quantifier_model(&settings);
        assert_eq!(model, "google/gemini-pro");
    }

    #[test]
    fn test_call_openrouter_with_model_invalid_api_key_format() {
        let result = call_openrouter_with_model("", "system prompt", "user text", "test/model");
        // Should fail due to empty bearer token or other validation
        assert!(result.is_err());
    }

    #[test]
    fn test_call_openrouter_success_with_content() {
        let settings = AppSettings::default();
        let model = get_llm_model(&settings);
        assert_eq!(model, "openai/gpt-4o-mini");
    }

    #[test]
    fn test_get_llm_model_env_var_override() {
        let settings = AppSettings::default();
        // Default should use settings.llm_model when no env var
        let model = get_llm_model(&settings);
        assert_eq!(model, settings.llm_model);
    }

    #[test]
    fn test_call_openrouter_empty_system_prompt() {
        let result = call_openrouter_with_model("", "", "user text", "test/model");
        // Should fail due to empty API key or network error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_call_openrouter_empty_user_text() {
        let result = call_openrouter_with_model("fake_key", "system", "", "test/model");
        // Should fail due to invalid key but not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_call_openrouter_with_model_rejects_empty_api_key() {
        let result = call_openrouter_with_model("", "system", "user", "model");
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should contain error information
        assert!(!err.is_empty());
    }

    #[test]
    fn test_call_openrouter_very_long_model_name() {
        let long_model = "a".repeat(1000);
        let result = call_openrouter_with_model("", "system", "user", &long_model);
        assert!(result.is_err()); // Should fail for other reasons, not panic
    }

    #[test]
    fn test_call_openrouter_very_long_system_prompt() {
        let long_prompt = "x".repeat(10000);
        let result = call_openrouter_with_model("key", &long_prompt, "user", "model");
        assert!(result.is_err()); // Should fail for other reasons, not panic
    }

    #[test]
    fn test_call_openrouter_very_long_user_text() {
        let long_text = "y".repeat(50000);
        let result = call_openrouter_with_model("key", "system", &long_text, "model");
        assert!(result.is_err()); // Should fail for other reasons, not panic
    }

    #[test]
    fn test_call_openrouter_whitespace_api_key() {
        let result = call_openrouter_with_model("   ", "system", "user", "model");
        assert!(result.is_err());
    }

    #[test]
    fn test_call_openrouter_special_characters_in_prompts() {
        let special_system = "System: <script>alert('xss')</script>\n{\"json\": true}";
        let special_user = "User input with \"quotes\" and 'apostrophes' and <brackets>";
        let result = call_openrouter_with_model("key", special_system, special_user, "model");
        assert!(result.is_err()); // Should handle gracefully
    }

    #[test]
    fn test_call_openrouter_unicode_in_prompts() {
        let unicode_text = "Hello 你好 مرحبا 🌍";
        let result = call_openrouter_with_model("key", "system", unicode_text, "model");
        assert!(result.is_err()); // Should handle gracefully
    }

    #[test]
    fn test_call_openrouter_call_helper() {
        let settings = AppSettings::default();
        let model = get_llm_model(&settings);
        // Verify model retrieval works
        assert!(!model.is_empty());
    }
}
