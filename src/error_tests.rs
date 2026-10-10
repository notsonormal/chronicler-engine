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

    let err = EngineError::Llm(LlmFailure::EmptyResponse {
        raw_response: String::new(),
    });
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
            EngineError::Llm(LlmFailure::EmptyResponse {
                raw_response: "{}".to_string(),
            }),
            "LLM Error: empty response",
        ),
        (
            EngineError::Llm(LlmFailure::TokenBudgetSpent {
                raw_response: "{}".to_string(),
                completion_tokens: Some(2048),
                reasoning_tokens: Some(2041),
            }),
            "LLM Error: the model spent its whole token budget before writing an answer (2048 completion tokens, 2041 of them reasoning)",
        ),
        (
            EngineError::Llm(LlmFailure::TokenBudgetSpent {
                raw_response: "{}".to_string(),
                completion_tokens: None,
                reasoning_tokens: None,
            }),
            "LLM Error: the model spent its whole token budget before writing an answer",
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
    let err: EngineError = InternalError::new("test invariant").into();
    match err {
        EngineError::Internal(InternalError { invariant }) => {
            assert_eq!(invariant, "test invariant");
        }
        other => panic!("Expected EngineError::Internal, got: {other:?}"),
    }
}

#[test]
fn test_token_budget_spent_display_counts_the_spend() {
    let err = LlmFailure::TokenBudgetSpent {
        raw_response: "{}".to_string(),
        completion_tokens: Some(2048),
        reasoning_tokens: Some(2041),
    };
    assert_eq!(
        err.to_string(),
        "LLM returned no answer after spending its whole token budget (2048 completion tokens, 2041 of them reasoning)"
    );

    let completion_only = LlmFailure::TokenBudgetSpent {
        raw_response: "{}".to_string(),
        completion_tokens: Some(300),
        reasoning_tokens: None,
    };
    assert_eq!(
        completion_only.to_string(),
        "LLM returned no answer after spending its whole token budget (300 completion tokens)"
    );

    let unreported = LlmFailure::TokenBudgetSpent {
        raw_response: "{}".to_string(),
        completion_tokens: None,
        reasoning_tokens: Some(12),
    };
    assert_eq!(
        unreported.to_string(),
        "LLM returned no answer after spending its whole token budget"
    );
}

#[test]
fn test_raw_response_body_is_present_for_the_body_carrying_kinds() {
    let empty = LlmFailure::EmptyResponse {
        raw_response: "{\"choices\":[]}".to_string(),
    };
    assert_eq!(empty.raw_response_body(), Some("{\"choices\":[]}"));

    let spent = LlmFailure::TokenBudgetSpent {
        raw_response: "{\"finish_reason\":\"length\"}".to_string(),
        completion_tokens: Some(2048),
        reasoning_tokens: None,
    };
    assert_eq!(
        spent.raw_response_body(),
        Some("{\"finish_reason\":\"length\"}")
    );

    let unparsed = LlmFailure::ParseError {
        raw_response: "not json".to_string(),
        expected_format: "valid JSON",
    };
    assert_eq!(unparsed.raw_response_body(), Some("not json"));

    assert_eq!(LlmFailure::Timeout.raw_response_body(), None);
    assert_eq!(
        LlmFailure::Network {
            url: "http://localhost:11434".to_string(),
            detail: "connection refused".to_string(),
        }
        .raw_response_body(),
        None
    );
    assert_eq!(
        LlmFailure::Http {
            status: 503,
            body: "unavailable".to_string(),
        }
        .raw_response_body(),
        None
    );
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
