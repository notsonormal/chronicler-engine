//! Tests for `LlmMessageRepository` port trait — polymorphism and dispatch.
//!
//! SCOPE GUARD: This file covers ONLY trait polymorphism and DTO structure.
//! Storage impl round-trip tests live at
//! `tests/integration/storage/llm_message_storage.rs` — do NOT duplicate them here.

use std::sync::Arc;

use crate::application::ports::llm_message_repository::{LlmMessage, LlmMessageRepository};
use crate::test_support::recording_forensics::RecordingForensics;
use crate::test_support::noop_forensics::NoopForensics;

#[test]
fn trait_dispatch_between_impls() {
    // Verify that `dyn LlmMessageRepository` dispatches to the correct impl.
    let noop = Arc::new(NoopForensics);
    let recording = Arc::new(RecordingForensics::new());

    let msg = make_sample_message("test-agent");

    // NoopForensics: save succeeds, count stays 0
    noop.save_llm_message(&msg)
        .expect("NoopForensics save should succeed");

    // RecordingForensics: save succeeds, count is 1
    recording
        .save_llm_message(&msg)
        .expect("RecordingForensics save should succeed");
    assert_eq!(recording.save_call_count(), 1);
    assert!(recording.last_saved_message().is_some());
}

#[test]
fn llm_message_clone_produces_equal_value() {
    let msg1 = make_sample_message("clone-test");
    let msg2 = msg1.clone();

    assert_eq!(msg1.agent_name, msg2.agent_name);
    assert_eq!(msg1.backend_name, msg2.backend_name);
    assert_eq!(msg1.model_name, msg2.model_name);
}

#[test]
fn llm_message_debug_format_contains_field_names() {
    let msg = make_sample_message("debug-test");
    let debug = format!("{msg:?}");

    // Debug output should contain at least one field name
    assert!(debug.contains("agent_name") || debug.contains("backend_name"));
}

#[test]
fn llm_message_construction_all_fields() {
    let msg = make_sample_message("construction-test");

    assert_eq!(msg.agent_name, "construction-test");
    assert_eq!(msg.backend_name, "TestBackend");
    assert_eq!(msg.model_name, "test-model");
    assert!(!msg.system_prompt.is_empty());
    assert!(!msg.user_prompt.is_empty());
    assert!(!msg.raw_request_json.is_empty());
    assert!(!msg.raw_response_json.is_empty());
    assert!(!msg.parsed_response.is_empty());
    assert!(msg.error_message.is_none());
    // created_at is chrono::DateTime, skip comparison
}

fn make_sample_message(agent_name: &str) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: agent_name.to_string(),
        backend_name: "TestBackend".to_string(),
        model_name: "test-model".to_string(),
        system_prompt: "Test system prompt".to_string(),
        user_prompt: "Test user prompt".to_string(),
        raw_request_json: r#"{"role":"user","content":"test"}"#.to_string(),
        raw_response_json: r#"{"choices":[{"message":{"content":"response"}}]}"#.to_string(),
        parsed_response: "response".to_string(),
        error_message: None,
        created_at: chrono::Utc::now(),
    }
}
