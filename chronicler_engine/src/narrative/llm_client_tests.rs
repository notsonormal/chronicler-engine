use crate::narrative::llm_client::{
    apply_gemma4_thinking_suffix, call_chat_completions, call_ollama,
    call_openrouter_with_model, extract_content_from_response, parse_chat_response,
    sanitize_llm_output,
};

// --- extract_content_from_response tests ---

#[test]
fn test_extract_content_from_content_field() {
    let json = serde_json::json!({
        "choices": [{ "message": { "content": "hello world" } }]
    });
    let result = extract_content_from_response(&json);
    assert_eq!(result, Some(("hello world".to_string(), "content")));
}

#[test]
fn test_extract_content_from_reasoning_field() {
    let json = serde_json::json!({
        "choices": [{ "message": { "reasoning": "because logic" } }]
    });
    let result = extract_content_from_response(&json);
    assert_eq!(result, Some(("because logic".to_string(), "reasoning")));
}

#[test]
fn test_extract_content_from_reasoning_content_field() {
    let json = serde_json::json!({
        "choices": [{ "message": { "reasoning_content": "deep thought" } }]
    });
    let result = extract_content_from_response(&json);
    assert_eq!(
        result,
        Some(("deep thought".to_string(), "reasoning_content"))
    );
}

#[test]
fn test_extract_content_prefers_content_over_reasoning() {
    let json = serde_json::json!({
        "choices": [{ "message": { "content": "primary", "reasoning": "secondary" } }]
    });
    let result = extract_content_from_response(&json);
    assert_eq!(result, Some(("primary".to_string(), "content")));
}

#[test]
fn test_extract_content_missing_message() {
    let json = serde_json::json!({ "choices": [{}] });
    let result = extract_content_from_response(&json);
    assert_eq!(result, None);
}

#[test]
fn test_extract_content_missing_choices() {
    let json = serde_json::json!({});
    let result = extract_content_from_response(&json);
    assert_eq!(result, None);
}

#[test]
fn test_extract_content_no_content_fields() {
    let json = serde_json::json!({
        "choices": [{ "message": { "role": "assistant" } }]
    });
    let result = extract_content_from_response(&json);
    assert_eq!(result, None);
}

// --- parse_chat_response tests ---

#[test]
fn test_parse_chat_response_success_content() {
    let raw = r#"{"choices":[{"message":{"content":"hello"}}]}"#;
    let result = parse_chat_response(raw, 1);
    assert_eq!(result, Ok("hello".to_string()));
}

#[test]
fn test_parse_chat_response_success_reasoning() {
    let raw = r#"{"choices":[{"message":{"reasoning":"think"}}]}"#;
    let result = parse_chat_response(raw, 1);
    assert_eq!(result, Ok("think".to_string()));
}

#[test]
fn test_parse_chat_response_api_error() {
    let raw = r#"{"error":{"message":"rate limited"}}"#;
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rate limited"));
}

#[test]
fn test_parse_chat_response_api_error_no_message() {
    let raw = r#"{"error":{}}"#;
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown API error"));
}

#[test]
fn test_parse_chat_response_missing_content() {
    let raw = r#"{"choices":[{"message":{"role":"assistant"}}]}"#;
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("parse error"));
}

#[test]
fn test_parse_chat_response_malformed_json() {
    let raw = "not json";
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse LLM response"));
}

#[test]
fn test_parse_chat_response_empty_json() {
    let raw = "{}";
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("parse error"));
}

// --- Integration-style tests for network calls ---

#[test]
fn test_call_openrouter_with_model_invalid_api_key_format() {
    let result = call_openrouter_with_model("", "system prompt", "user text", "test/model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_empty_system_prompt() {
    let result = call_openrouter_with_model("", "", "user text", "test/model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_empty_user_text() {
    let result = call_openrouter_with_model("fake_key", "system", "", "test/model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_with_model_rejects_empty_api_key() {
    let result = call_openrouter_with_model("", "system", "user", "model", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn test_call_openrouter_very_long_model_name() {
    let long_model = "a".repeat(1000);
    let result = call_openrouter_with_model("", "system", "user", &long_model, None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_very_long_system_prompt() {
    let long_prompt = "x".repeat(10000);
    let result = call_openrouter_with_model("key", &long_prompt, "user", "model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_very_long_user_text() {
    let long_text = "y".repeat(50000);
    let result = call_openrouter_with_model("key", "system", &long_text, "model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_whitespace_api_key() {
    let result = call_openrouter_with_model("   ", "system", "user", "model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_special_characters_in_prompts() {
    let special_system = "System: <script>alert('xss')</script>\n{\"json\": true}";
    let special_user = "User input with \"quotes\" and 'apostrophes' and <brackets>";
    let result = call_openrouter_with_model("key", special_system, special_user, "model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_openrouter_unicode_in_prompts() {
    let unicode_text = "Hello 你好 مرحبا 🌍";
    let result = call_openrouter_with_model("key", "system", unicode_text, "model", None);
    assert!(result.is_err());
}

#[test]
fn test_call_ollama_invalid_url() {
    // call_ollama with a fake URL should fail gracefully without panic
    let result = call_ollama("http://localhost:59999", "model", "system", "user", None);
    assert!(result.is_err());
}

#[test]
fn test_call_ollama_empty_system_prompt() {
    // call_ollama with empty system prompt should not panic
    let result = call_ollama("http://localhost:59999", "model", "", "user message", None);
    assert!(result.is_err());
}

// --- sanitize_llm_output tests ---

#[test]
fn test_sanitize_leading_channel_close() {
    let input = "<channel|>The heavy iron gates...";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "The heavy iron gates...");
}

#[test]
fn test_sanitize_thought_block() {
    let input = "<thought>The user wants to continue...</thought>The gates creaked.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "The gates creaked.");
}

#[test]
fn test_sanitize_channel_thought_block() {
    let input = "<|channel>thought\nSome reasoning here\n<channel|>Narrative text.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "Narrative text.");
}

#[test]
fn test_sanitize_orphan_turn_markers() {
    let input = "<|turn>modelStart of text<turn|>more text<|turn>end.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "Start of textmore textend.");
}

#[test]
fn test_sanitize_combined_artifacts() {
    let input = "<channel|><thought>reasoning</thought><|turn>modelThe real content.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "The real content.");
}

#[test]
fn test_sanitize_clean_text_unchanged() {
    let input = "The heavy iron gates offered no resistance.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, input);
}

#[test]
fn test_sanitize_empty_string() {
    assert_eq!(sanitize_llm_output(""), "");
}

#[test]
fn test_sanitize_whitespace_only() {
    assert_eq!(sanitize_llm_output("   "), "");
}

#[test]
fn test_sanitize_multiple_thought_blocks() {
    let input = "<thought>first</thought>A<thought>second</thought>B";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "AB");
}

#[test]
fn test_sanitize_paragraph_indentation() {
    // Models often emit indented paragraphs that pulldown-cmark
    // treats as code blocks (<pre><code>), causing bubble overflow.
    let input = "  First paragraph.\n\n        Second paragraph.\n\n        Third paragraph.";
    let result = sanitize_llm_output(input);
    assert_eq!(
        result,
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph."
    );
}

// --- apply_gemma4_thinking_suffix tests ---

#[test]
fn test_gemma4_suffix_applied_for_gemma4_name() {
    let input = "User prompt";
    let result = apply_gemma4_thinking_suffix(input, "gemma4:latest");
    assert!(result.contains("<|turn>model"));
    assert!(result.contains("<|channel>thought"));
    assert!(result.contains("<channel|>"));
    assert!(!result.contains("\n<turn|>\n")); // old malformed format
}

#[test]
fn test_gemma4_suffix_applied_for_gemma_dash() {
    let input = "User prompt";
    let result = apply_gemma4_thinking_suffix(input, "mradermacher/gemma-4-26b");
    assert!(result.contains("<|turn>model"));
}

#[test]
fn test_gemma4_suffix_not_applied_for_other_models() {
    let input = "User prompt";
    let result = apply_gemma4_thinking_suffix(input, "llama3:8b");
    assert_eq!(result, input);
}

// --- Mock HTTP server tests for call_chat_completions ---

#[test]
fn test_call_chat_completions_mock_server_success() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let _request = String::from_utf8_lossy(&buf[..n]);

        let body = r#"{"choices":[{"message":{"content":"mocked narration"}}]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        let _ = stream.flush();
    });

    let result = call_chat_completions(
        &format!("http://127.0.0.1:{port}"),
        Some("test-key"),
        "test-model",
        "system prompt",
        "user prompt",
        Some("Test Title"),
        Some(512),
    );
    assert_eq!(result, Ok("mocked narration".to_string()));
}

#[test]
fn test_call_chat_completions_mock_server_error_status() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf).unwrap_or(0);

        let body = r#"{"error":"invalid request"}"#;
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        let _ = stream.flush();
    });

    let result = call_chat_completions(
        &format!("http://127.0.0.1:{port}"),
        None,
        "test-model",
        "",
        "user",
        None,
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("400") || err.contains("Error communicating"),
        "Expected API error, got: {err}"
    );
}
