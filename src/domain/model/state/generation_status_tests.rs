use crate::domain::model::agent::Kind;
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind, Record};
use crate::error::{EngineError, InternalError, LlmFailure, NarrativeFailure};

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
fn test_unlisted_error_is_other() {
    assert_eq!(
        kind_of(EngineError::Validation("no anchor".to_string())),
        GenerationFailureKind::Other
    );
}

#[test]
fn test_an_invalid_room_names_the_room() {
    let failure = GenerationFailure::invalid_room("cellar");
    assert_eq!(failure.kind, GenerationFailureKind::InvalidRoom);
    assert_eq!(failure.line(), "The room cellar is not on the map.");
    assert_eq!(failure.raw, "cellar");
}

#[test]
fn record_names() {
    let rows = [
        (Record::Game, "the game"),
        (Record::Persona, "the persona"),
        (Record::Story, "the story"),
        (Record::Message, "the message"),
        (Record::StartState, "the start state"),
        (Record::SavedState, "the saved state"),
        (Record::CurrentState, "the current state"),
        (Record::World, "the world"),
        (Record::Settings, "the settings"),
        (Record::Preset, "the preset"),
    ];
    let error = EngineError::Internal(InternalError::new("the row cannot be read"));
    for (record, name) in rows {
        let line = format!("The data for this request is missing: {name}.");
        let miss = GenerationFailure::missing(record);
        assert_eq!(miss.kind, GenerationFailureKind::MissingData);
        assert_eq!(miss.line(), line);
        assert_eq!(
            miss.raw, name,
            "a miss has no error: the disclosure holds the name"
        );

        let unreadable = GenerationFailure::unreadable(record, &error);
        assert_eq!(
            unreadable.line(),
            line,
            "a read error shows the line of a miss"
        );
        assert_eq!(unreadable.raw, error.to_string());
    }
}

#[test]
fn failure_of_event() {
    let inner = GenerationFailure::new(GenerationFailureKind::Unreachable, "connection refused");

    let wrapped = inner.clone().of_generation(Kind::Event);
    assert_eq!(wrapped.kind, GenerationFailureKind::EventNarrationFailed);
    assert_eq!(wrapped.line(), "The event narration failed.");
    assert_eq!(
        wrapped.raw, "The language model could not be reached.",
        "the line of the inner failure is in the raw text"
    );

    for kind in [Kind::Narration, Kind::Impersonate, Kind::UserRegen] {
        assert_eq!(
            inner.clone().of_generation(kind),
            inner,
            "{kind:?} keeps its line"
        );
    }
}

#[test]
fn every_failure_kind_has_a_line() {
    for kind in GenerationFailureKind::ALL {
        assert!(!kind.line("the story").is_empty(), "{kind:?}");
    }
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
