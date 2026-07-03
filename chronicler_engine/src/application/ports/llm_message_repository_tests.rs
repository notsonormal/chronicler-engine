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
