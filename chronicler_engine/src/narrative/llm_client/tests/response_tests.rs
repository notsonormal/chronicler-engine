use crate::narrative::llm_client::response::{extract_content_from_response, parse_chat_response};

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
