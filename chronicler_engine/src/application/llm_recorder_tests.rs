//! Unit tests for `LlmCallRecorder` orchestrator.
//!
//! Tests verify the orchestration logic: provider gets called, sanitization
//! runs, forensics are persisted, errors propagate.

use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::error::EngineError;
use crate::test_support::recording_forensics::RecordingForensics;

const _: fn() = || {
    fn assert<T: Send + Sync>() {}
    assert::<LlmCallRecorder>();
};

#[test]
fn complete_happy_path_calls_provider_and_persists_forensics() {
    let provider = Arc::new(MockBackend::new());
    let forensics = Arc::new(RecordingForensics::new());
    let recorder = LlmCallRecorder::new(provider, forensics.clone());

    let result = recorder
        .complete("narrator", "system prompt", "user prompt", None)
        .expect("complete should succeed");

    // Provider was called and returned text
    assert!(!result.text.is_empty());

    // Forensics persisted exactly one message
    assert_eq!(forensics.save_call_count(), 1);
    let saved = forensics
        .last_saved_message()
        .expect("message should be saved");

    // Message has correct metadata
    assert_eq!(saved.agent_name, "narrator");
    assert_eq!(saved.backend_name, "Mock");
    assert_eq!(saved.model_name, "mock");
    assert_eq!(saved.system_prompt, "system prompt");
    assert_eq!(saved.user_prompt, "user prompt");
}

#[test]
fn complete_strips_thought_tags_from_parsed_response() {
    // MockBackend echoes user prompt in its response. We'll inject a thought tag.
    let provider = Arc::new(MockBackend::new().with_narrations(vec![
        "<thought>inner monologue</thought>Hello, user!".to_string(),
    ]));
    let forensics = Arc::new(RecordingForensics::new());
    let recorder = LlmCallRecorder::new(provider, forensics.clone());

    let result = recorder
        .complete("narrator", "system", "user", None)
        .expect("complete should succeed");

    // Sanitized result text should NOT contain the thought tag
    assert!(!result.text.contains("<thought>"));
    assert!(result.text.contains("Hello, user!"));

    // Raw response JSON in forensics IS the original (unsanitized)
    let saved = forensics
        .last_saved_message()
        .expect("message should be saved");
    assert!(saved.raw_response_json.contains("<thought>"));

    // Parsed response in forensics IS sanitized
    assert!(!saved.parsed_response.contains("<thought>"));
    assert!(saved.parsed_response.contains("Hello, user!"));
}

#[test]
fn complete_propagates_provider_error_without_forensics_write() {
    let provider = Arc::new(MockBackend::new().with_fail());
    let forensics = Arc::new(RecordingForensics::new());
    let recorder = LlmCallRecorder::new(provider, forensics.clone());

    let err = recorder
        .complete("narrator", "system", "user", None)
        .unwrap_err();

    // Error propagates (MockBackend uses Narrative error variant)
    assert!(matches!(err, EngineError::Narrative(_)));

    // No forensics write happened
    assert_eq!(forensics.save_call_count(), 0);
}

#[test]
fn complete_propagates_forensics_error_after_provider_success() {
    let provider = Arc::new(MockBackend::new());
    let forensics = Arc::new(
        RecordingForensics::new().with_next_save_error(EngineError::Io("disk full".into())),
    );
    let recorder = LlmCallRecorder::new(provider, forensics.clone());

    let err = recorder
        .complete("narrator", "system", "user", None)
        .unwrap_err();

    // Error propagates from forensics layer
    assert!(matches!(err, EngineError::Io(_)));
    // Error message preserved
    assert!(err.to_string().contains("disk full"));
}

#[test]
fn provider_accessor_returns_injected_provider() {
    let original: Arc<dyn LlmProvider> = Arc::new(MockBackend::new());
    let forensics = Arc::new(RecordingForensics::new());
    let recorder = LlmCallRecorder::new(original.clone(), forensics);

    assert!(Arc::ptr_eq(&original, recorder.provider()));
}

#[test]
fn recorder_with_configurable_mock_backend() {
    // Verify that various MockBackend configurations work through the recorder
    let mocking_empty = Arc::new(MockBackend::new().with_empty_response());
    let forensics = Arc::new(RecordingForensics::new());
    let recorder = LlmCallRecorder::new(mocking_empty, forensics.clone());

    let result = recorder.complete("narrator", "sys", "user", None).unwrap();
    assert_eq!(result.text, "");
    assert_eq!(forensics.save_call_count(), 1);
}
