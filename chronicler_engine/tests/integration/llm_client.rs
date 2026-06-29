//! Integration tests for LLM client HTTP communication
//!
//! These tests spawn real TCP servers to validate HTTP request/response handling
//! without making external network calls.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use chronicler_engine::adapters::driven::llm::transport::call_chat_completions;

// --- Mock HTTP server tests for call_chat_completions ---

#[test]
fn test_call_chat_completions_mock_server_success() {
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
