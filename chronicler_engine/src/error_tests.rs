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
