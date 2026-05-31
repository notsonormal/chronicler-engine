use crate::narrative::llm_client::{
    build_request_payload, call_chat_completions, call_ollama, call_openrouter_with_model,
    extract_content_from_response, parse_chat_response,
};
// --- build_request_payload tests (extracted function) ---
#[test]
fn test_build_request_payload_empty_system_prompt_omits_system_message() {
    // Empty system prompt should result in only user message
    let (payload, _raw_json) = build_request_payload("test-model", "", "user question", 1024);
    let messages = payload["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1, "Should have only user message");
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "user question");
    assert_eq!(payload["model"], "test-model");
    assert_eq!(payload["max_tokens"], 1024);
    assert_eq!(payload["stream"], false);
}
#[test]
fn test_build_request_payload_nonempty_system_prompt_includes_both_messages() {
    // Non-empty system prompt should include both system and user messages
    let (payload, _raw_json) =
        build_request_payload("test-model", "system instruction", "user question", 2048);
    let messages = payload["messages"].as_array().unwrap();
    assert_eq!(
        messages.len(),
        2,
        "Should have both system and user messages"
    );
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[0]["content"], "system instruction");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "user question");
    assert_eq!(payload["model"], "test-model");
    assert_eq!(payload["max_tokens"], 2048);
}
#[test]
fn test_build_request_payload_max_tokens_correctly_serialized() {
    // max_tokens should be serialized as integer, not string
    let (payload, raw_json) = build_request_payload("gpt-4", "be helpful", "what is 2+2?", 512);
    assert_eq!(payload["max_tokens"], 512);
    assert!(raw_json.contains(r#""max_tokens":512"#) || raw_json.contains(r#""max_tokens": 512"#));
    // Verify JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    assert_eq!(parsed["max_tokens"], 512);
}
// --- handle_response tests (extracted function) ---
// Note: These tests require implementing handle_response() first
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
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_parse_chat_response_success_reasoning() {
    let raw = r#"{"choices":[{"message":{"reasoning":"think"}}]}"#;
    let result = parse_chat_response(raw, 1);
    assert_eq!(result.unwrap(), "think");
}

#[test]
fn test_parse_chat_response_api_error() {
    let raw = r#"{"error":{"message":"rate limited"}}"#;
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("rate limited"));
}

#[test]
fn test_parse_chat_response_api_error_no_message() {
    let raw = r#"{"error":{}}"#;
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unknown API error")
    );
}

#[test]
fn test_parse_chat_response_missing_content() {
    let raw = r#"{"choices":[{"message":{"role":"assistant"}}]}"#;
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse LLM response")
    );
}

#[test]
fn test_parse_chat_response_malformed_json() {
    let raw = "not json";
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse LLM response")
    );
}

#[test]
fn test_parse_chat_response_empty_json() {
    let raw = "{}";
    let result = parse_chat_response(raw, 1);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse LLM response")
    );
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
    assert!(matches!(err, crate::error::EngineError::Llm(_)));
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
    assert_eq!(result.unwrap().text, "mocked narration");
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
    let msg = err.to_string();
    assert!(
        msg.contains("400") || msg.contains("Error communicating"),
        "Expected API error, got: {msg}"
    );
}

#[test]
fn test_call_chat_completions_mock_server_no_content() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf).unwrap_or(0);

        let body = r#"{"choices":[{"message":{"role":"assistant"}}]}"#;
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
        None,
        "test-model",
        "system",
        "user",
        None,
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("parse") || err.contains("Failed to parse"),
        "Expected parse error, got: {err}"
    );
}

#[test]
fn test_call_chat_completions_mock_server_truncated_body() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf).unwrap_or(0);

        // Claim a huge body but send nothing, forcing reqwest to fail reading
        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 99999\r\nConnection: close\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        let _ = stream.flush();
    });

    let result = call_chat_completions(
        &format!("http://127.0.0.1:{port}"),
        None,
        "test-model",
        "system",
        "user",
        None,
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("read") || err.contains("body") || err.contains("network"),
        "Expected body read error, got: {err}"
    );
}
