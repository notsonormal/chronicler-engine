//! Tests for the `RecordingForensics` spy.

use chrono::Utc;

use crate::application::ports::llm_message_repository::{LlmMessage, LlmMessageRepository};
use crate::error::EngineError;
use crate::test_support::recording_forensics::RecordingForensics;

fn sample_message(agent_name: &str) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: agent_name.to_string(),
        backend_name: "MockBackend".to_string(),
        model_name: "mock-model".to_string(),
        system_prompt: "system".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "{}".to_string(),
        raw_response_json: "{\"text\":\"hi\"}".to_string(),
        parsed_response: "hi".to_string(),
        error_message: None,
        created_at: Utc::now(),
    }
}

#[test]
fn save_increments_count_and_captures_message() {
    let spy = RecordingForensics::new();

    assert_eq!(spy.save_call_count(), 0);
    assert!(spy.last_saved_message().is_none());

    spy.save_llm_message(&sample_message("A")).unwrap();
    spy.save_llm_message(&sample_message("B")).unwrap();

    assert_eq!(spy.save_call_count(), 2);
    let last = spy.last_saved_message().expect("expected captured message");
    assert_eq!(last.agent_name, "B");
}

#[test]
fn next_save_error_is_returned_once_then_cleared() {
    let spy = RecordingForensics::new().with_next_save_error(EngineError::Io("boom".into()));

    let err = spy.save_llm_message(&sample_message("A")).unwrap_err();
    assert!(matches!(err, EngineError::Io(_)));

    // Error cleared after first use; subsequent saves succeed.
    spy.save_llm_message(&sample_message("B")).unwrap();
    // Both calls attempted (one errored, one succeeded); counter increments on entry.
    assert_eq!(spy.save_call_count(), 2);
}

#[test]
fn list_returns_configured_response_independent_of_saves() {
    let configured = vec![sample_message("X"), sample_message("Y")];
    let spy = RecordingForensics::new().with_list_response(configured.clone());

    spy.save_llm_message(&sample_message("A")).unwrap();

    let listed = spy.list_latest_llm_messages(10).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].agent_name, "X");
    assert_eq!(listed[1].agent_name, "Y");
}
