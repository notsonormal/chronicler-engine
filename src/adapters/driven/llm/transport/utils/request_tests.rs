use crate::adapters::driven::llm::transport::utils::request::build_request_payload;

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
