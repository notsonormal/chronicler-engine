use crate::narrative::llm_client::{call_chat_completions, call_ollama, call_openrouter_with_model};

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
