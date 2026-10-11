use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind};
use crate::error::{EngineError, LlmFailure, NarrativeFailure};

fn kind_of(error: EngineError) -> GenerationFailureKind {
    GenerationFailure::from_engine_error(&error).kind
}

#[test]
fn test_timeout_is_unreachable() {
    assert_eq!(
        kind_of(EngineError::Llm(LlmFailure::Timeout)),
        GenerationFailureKind::Unreachable
    );
}

#[test]
fn test_network_failure_is_unreachable() {
    assert_eq!(
        kind_of(EngineError::Llm(LlmFailure::Network {
            url: "http://localhost:11434".to_string(),
            detail: "connection refused".to_string(),
        })),
        GenerationFailureKind::Unreachable
    );
}

#[test]
fn test_http_status_failure_is_unreachable() {
    assert_eq!(
        kind_of(EngineError::Llm(LlmFailure::Http {
            status: 503,
            body: "unavailable".to_string(),
        })),
        GenerationFailureKind::Unreachable
    );
}

#[test]
fn test_context_overflow_is_prompt_too_long() {
    assert_eq!(
        kind_of(EngineError::ContextOverflow {
            requested: 9000,
            max: 8000,
        }),
        GenerationFailureKind::PromptTooLong
    );
}

#[test]
fn test_prompt_build_failure_is_prompt_too_long() {
    assert_eq!(
        kind_of(EngineError::Narrative(NarrativeFailure::PromptBuild {
            stage: "narration",
            reason: "budget",
        })),
        GenerationFailureKind::PromptTooLong
    );
}

#[test]
fn test_parse_error_is_unreadable_answer() {
    assert_eq!(
        kind_of(EngineError::Llm(LlmFailure::ParseError {
            raw_response: "not json".to_string(),
            expected_format: "JSON",
        })),
        GenerationFailureKind::UnreadableAnswer
    );
}

#[test]
fn test_empty_response_is_unreadable_answer() {
    assert_eq!(
        kind_of(EngineError::Llm(LlmFailure::EmptyResponse {
            raw_response: "{}".to_string(),
        })),
        GenerationFailureKind::UnreadableAnswer
    );
}

#[test]
fn test_spent_token_budget_is_its_own_kind() {
    assert_eq!(
        kind_of(EngineError::Llm(LlmFailure::TokenBudgetSpent {
            raw_response: "{}".to_string(),
            completion_tokens: Some(2048),
            reasoning_tokens: Some(2041),
        })),
        GenerationFailureKind::TokenBudgetSpent
    );
}

#[test]
fn test_narration_generation_failure_is_unreadable_answer() {
    assert_eq!(
        kind_of(EngineError::Narrative(NarrativeFailure::Generation {
            stage: "narration",
            reason: "empty",
        })),
        GenerationFailureKind::UnreadableAnswer
    );
}

#[test]
fn test_io_failure_is_save_failed() {
    assert_eq!(
        kind_of(EngineError::Io("disk full".to_string())),
        GenerationFailureKind::SaveFailed
    );
}

#[test]
fn test_database_failure_is_save_failed() {
    assert_eq!(
        kind_of(EngineError::Database(rusqlite::Error::QueryReturnedNoRows)),
        GenerationFailureKind::SaveFailed
    );
}

#[test]
fn test_room_not_found_is_scene_missing() {
    assert_eq!(
        kind_of(EngineError::RoomNotFound("cellar".to_string())),
        GenerationFailureKind::SceneMissing
    );
}

#[test]
fn test_missing_preset_is_preset_missing() {
    assert_eq!(
        kind_of(EngineError::PresetNotFound("system_default".to_string())),
        GenerationFailureKind::PresetMissing
    );
}

#[test]
fn test_unlisted_error_is_other() {
    assert_eq!(
        kind_of(EngineError::Validation("no anchor".to_string())),
        GenerationFailureKind::Other
    );
}

#[test]
fn test_raw_text_is_the_classified_error_message() {
    let failure = GenerationFailure::from_engine_error(&EngineError::RoomNotFound("cellar".into()));
    assert_eq!(failure.raw, "Room not found: cellar");
}

#[test]
fn test_with_context_prefixes_the_raw_text() {
    let failure = GenerationFailure::from_engine_error(&EngineError::Io("disk full".into()))
        .with_context("Trigger error");
    assert_eq!(failure.raw, "Trigger error: I/O error: disk full");
    assert_eq!(failure.kind, GenerationFailureKind::SaveFailed);
}

#[test]
fn test_a_named_failure_carries_its_kind_and_raw_text() {
    let failure = GenerationFailure::new(
        GenerationFailureKind::Other,
        "Retry failed: no anchor message",
    );
    assert_eq!(failure.kind, GenerationFailureKind::Other);
    assert_eq!(failure.raw, "Retry failed: no anchor message");
}
