use crate::error::{EngineError, InternalError, LlmFailure, NarrativeFailure};

#[test]
fn test_engine_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let engine_err: EngineError = io_err.into();
    match engine_err {
        EngineError::Io(msg) => {
            assert!(
                msg.contains("file not found"),
                "Expected error message to contain 'file not found', got: {msg}"
            );
        }
        other => panic!("Expected EngineError::Io, got: {other:?}"),
    }
}

#[test]
fn test_engine_error_display_variants() {
    let err = EngineError::Io("disk full".to_string());
    assert!(err.to_string().contains("disk full"));

    let err = EngineError::Parse("bad json".to_string());
    assert!(err.to_string().contains("bad json"));

    let err = EngineError::Llm(LlmFailure::EmptyResponse);
    assert!(err.to_string().contains("empty response"));

    let err = EngineError::ContextOverflow {
        requested: 100,
        max: 50,
    };
    assert!(err.to_string().contains("100"));
    assert!(err.to_string().contains("50"));

    let err = EngineError::WorldHasGames { game_count: 3 };
    assert!(err.to_string().contains("3"));
    assert!(err.to_string().contains("games"));
}

#[test]
fn test_llm_error_string_maps_user_facing_messages() {
    let cases = [
        (
            EngineError::Llm(LlmFailure::Timeout),
            "LLM Error: request timed out",
        ),
        (
            EngineError::Llm(LlmFailure::Network {
                url: "http://localhost:11434".to_string(),
                detail: "connection refused".to_string(),
            }),
            "LLM Error: network error (http://localhost:11434) — connection refused",
        ),
        (
            EngineError::Llm(LlmFailure::ParseError {
                raw_response: "invalid".to_string(),
                expected_format: "JSON",
            }),
            "LLM Error: unexpected response format (expected JSON)",
        ),
        (
            EngineError::Llm(LlmFailure::EmptyResponse),
            "LLM Error: empty response",
        ),
        (
            EngineError::Llm(LlmFailure::Http {
                status: 503,
                body: "unavailable".to_string(),
            }),
            "LLM Error: HTTP 503 — unavailable",
        ),
        (
            EngineError::Narrative(NarrativeFailure::PromptBuild {
                stage: "assembly",
                reason: "budget",
            }),
            "LLM Error: Prompt build failed at stage 'assembly': budget",
        ),
        (
            EngineError::Config("missing preset".to_string()),
            "LLM Error: Configuration error: missing preset",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.llm_error_string(), expected);
    }
}

#[test]
fn test_internal_error_from_helper() {
    let err: EngineError = crate::error::internal_error("test invariant").into();
    match err {
        EngineError::Internal(InternalError { invariant }) => {
            assert_eq!(invariant, "test invariant");
        }
        other => panic!("Expected EngineError::Internal, got: {other:?}"),
    }
}

#[test]
fn test_llm_failure_display() {
    let err = LlmFailure::Http {
        status: 500,
        body: "server error".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("500"), "Expected status in message: {msg}");
    assert!(
        msg.contains("server error"),
        "Expected body in message: {msg}"
    );
}

#[test]
fn test_narrative_failure_display() {
    let err = NarrativeFailure::Generation {
        stage: "test",
        reason: "failed",
    };
    let msg = err.to_string();
    assert!(msg.contains("test"), "Expected stage in message: {msg}");
    assert!(msg.contains("failed"), "Expected reason in message: {msg}");
}

#[test]
fn test_llm_failure_into_engine_error() {
    let llm_err = LlmFailure::Timeout;
    let engine_err: EngineError = llm_err.into();
    match engine_err {
        EngineError::Llm(LlmFailure::Timeout) => {}
        other => panic!("Expected EngineError::Llm(Timeout), got: {other:?}"),
    }
}

#[test]
fn test_narrative_failure_into_engine_error() {
    let nar_err = NarrativeFailure::PromptBuild {
        stage: "test",
        reason: "budget",
    };
    let engine_err: EngineError = nar_err.into();
    match engine_err {
        EngineError::Narrative(NarrativeFailure::PromptBuild { stage, reason }) => {
            assert_eq!(stage, "test");
            assert_eq!(reason, "budget");
        }
        other => panic!("Expected EngineError::Narrative(PromptBuild), got: {other:?}"),
    }
}

#[test]
fn test_engine_error_display_not_found_variants() {
    let err = EngineError::GameNotFound(42u64);
    assert!(err.to_string().contains("Game not found: 42"), "got: {err}");

    let err = EngineError::PersonaNotFound("alice".to_string());
    assert!(
        err.to_string().contains("Persona not found: alice"),
        "got: {err}"
    );

    let err = EngineError::WorldNotFound("bob".to_string());
    assert!(
        err.to_string().contains("World not found: bob"),
        "got: {err}"
    );

    let err = EngineError::MessageNotFound(7u64);
    assert!(
        err.to_string().contains("Message not found: 7"),
        "got: {err}"
    );
}
