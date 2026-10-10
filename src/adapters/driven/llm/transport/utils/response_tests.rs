//! Unit tests for the transport's "what counts as an answer" rule.

use crate::adapters::driven::llm::transport::utils::response::parse_chat_response;
use crate::error::{EngineError, LlmFailure};

fn empty_response_body(error: EngineError) -> String {
    match error {
        EngineError::Llm(LlmFailure::EmptyResponse { raw_response }) => raw_response,
        other => panic!("expected EmptyResponse, got: {other:?}"),
    }
}

fn token_budget_spend(error: EngineError) -> (String, Option<u64>, Option<u64>) {
    match error {
        EngineError::Llm(LlmFailure::TokenBudgetSpent {
            raw_response,
            completion_tokens,
            reasoning_tokens,
        }) => (raw_response, completion_tokens, reasoning_tokens),
        other => panic!("expected TokenBudgetSpent, got: {other:?}"),
    }
}

#[test]
fn test_parse_chat_response_success_content() {
    let raw = r#"{"choices":[{"message":{"content":"hello"}}]}"#;
    let result = parse_chat_response(raw, 1);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_parse_chat_response_keeps_content_verbatim() {
    let raw = r#"{"choices":[{"message":{"content":"  padded answer  "}}]}"#;
    assert_eq!(parse_chat_response(raw, 1).unwrap(), "  padded answer  ");
}

#[test]
fn test_null_content_with_reasoning_is_an_empty_response() {
    let raw = r#"{"choices":[{"message":{"content":null,"reasoning":"let me think about this"},"finish_reason":"stop"}]}"#;
    let body = empty_response_body(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw, "the failure must carry the raw body");
}

#[test]
fn test_null_content_with_reasoning_content_is_an_empty_response() {
    let raw = r#"{"choices":[{"message":{"content":null,"reasoning_content":"deep thought"},"finish_reason":"stop"}]}"#;
    let body = empty_response_body(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw);
}

#[test]
fn test_empty_string_content_is_an_empty_response() {
    let raw = r#"{"choices":[{"message":{"content":""},"finish_reason":"stop"}]}"#;
    let body = empty_response_body(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw);
}

#[test]
fn test_whitespace_only_content_is_an_empty_response() {
    let raw =
        "{\"choices\":[{\"message\":{\"content\":\"  \\n\\t \"},\"finish_reason\":\"stop\"}]}";
    let body = empty_response_body(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw);
}

#[test]
fn test_missing_content_is_an_empty_response() {
    let raw = r#"{"choices":[{"message":{"role":"assistant"},"finish_reason":"stop"}]}"#;
    let body = empty_response_body(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw);
}

#[test]
fn test_length_finish_reason_with_empty_content_is_a_spent_token_budget() {
    let raw = r#"{
        "choices": [{"message": {"content": null, "reasoning": "thinking..."}, "finish_reason": "length"}],
        "usage": {"completion_tokens": 2048, "completion_tokens_details": {"reasoning_tokens": 2041}}
    }"#;
    let (body, completion_tokens, reasoning_tokens) =
        token_budget_spend(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw);
    assert_eq!(completion_tokens, Some(2048));
    assert_eq!(reasoning_tokens, Some(2041));
}

#[test]
fn test_length_finish_reason_without_usage_reports_no_counts() {
    let raw = r#"{"choices":[{"message":{"content":null},"finish_reason":"length"}]}"#;
    let (body, completion_tokens, reasoning_tokens) =
        token_budget_spend(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(body, raw);
    assert_eq!(completion_tokens, None);
    assert_eq!(reasoning_tokens, None);
}

#[test]
fn test_length_finish_reason_with_usage_but_no_reasoning_details() {
    let raw = r#"{
        "choices": [{"message": {"content": ""}, "finish_reason": "length"}],
        "usage": {"completion_tokens": 300}
    }"#;
    let (_, completion_tokens, reasoning_tokens) =
        token_budget_spend(parse_chat_response(raw, 1).unwrap_err());
    assert_eq!(completion_tokens, Some(300));
    assert_eq!(reasoning_tokens, None);
}

#[test]
fn test_length_finish_reason_with_content_is_an_answer() {
    let raw = r#"{"choices":[{"message":{"content":"the door creaks"},"finish_reason":"length"}]}"#;
    assert_eq!(parse_chat_response(raw, 1).unwrap(), "the door creaks");
}

#[test]
fn test_missing_finish_reason_with_empty_content_is_an_empty_response() {
    let raw = r#"{"choices":[{"message":{"content":null}}]}"#;
    assert!(matches!(
        parse_chat_response(raw, 1).unwrap_err(),
        EngineError::Llm(LlmFailure::EmptyResponse { .. })
    ));
}

#[test]
fn test_missing_choices_is_a_parse_error() {
    let raw = r#"{"id":"gen-1"}"#;
    match parse_chat_response(raw, 1).unwrap_err() {
        EngineError::Llm(LlmFailure::ParseError { raw_response, .. }) => {
            assert_eq!(raw_response, raw);
        }
        other => panic!("expected ParseError, got: {other:?}"),
    }
}

#[test]
fn test_empty_choices_is_a_parse_error() {
    let raw = r#"{"choices":[]}"#;
    assert!(matches!(
        parse_chat_response(raw, 1).unwrap_err(),
        EngineError::Llm(LlmFailure::ParseError { .. })
    ));
}

#[test]
fn test_missing_message_is_a_parse_error() {
    let raw = r#"{"choices":[{"finish_reason":"stop"}]}"#;
    assert!(matches!(
        parse_chat_response(raw, 1).unwrap_err(),
        EngineError::Llm(LlmFailure::ParseError { .. })
    ));
}

#[test]
fn test_null_message_is_a_parse_error() {
    let raw = r#"{"choices":[{"message":null}]}"#;
    assert!(matches!(
        parse_chat_response(raw, 1).unwrap_err(),
        EngineError::Llm(LlmFailure::ParseError { .. })
    ));
}

#[test]
fn test_malformed_json_is_a_parse_error() {
    let raw = "not json";
    match parse_chat_response(raw, 1).unwrap_err() {
        EngineError::Llm(LlmFailure::ParseError {
            raw_response,
            expected_format,
        }) => {
            assert_eq!(raw_response, raw);
            assert_eq!(expected_format, "valid JSON");
        }
        other => panic!("expected ParseError, got: {other:?}"),
    }
}

#[test]
fn test_empty_json_object_is_a_parse_error() {
    assert!(matches!(
        parse_chat_response("{}", 1).unwrap_err(),
        EngineError::Llm(LlmFailure::ParseError { .. })
    ));
}

#[test]
fn test_whitespace_only_input_is_a_parse_error() {
    // `parse_chat_response` trims leading whitespace, so whitespace-only input
    // reaches the malformed-JSON path rather than the empty-content one.
    assert!(matches!(
        parse_chat_response("   \n\t  ", 1).unwrap_err(),
        EngineError::Llm(LlmFailure::ParseError { .. })
    ));
}

#[test]
fn test_api_error_body_is_an_http_failure() {
    let raw = r#"{"error":{"message":"rate limited"}}"#;
    match parse_chat_response(raw, 1).unwrap_err() {
        EngineError::Llm(LlmFailure::Http { status, body }) => {
            assert_eq!(status, 200);
            assert_eq!(body, "rate limited");
        }
        other => panic!("expected Http, got: {other:?}"),
    }
}

#[test]
fn test_api_error_without_message_names_the_unknown_error() {
    let raw = r#"{"error":{}}"#;
    let err = parse_chat_response(raw, 1).unwrap_err();
    assert!(err.to_string().contains("Unknown API error"), "{err}");
}
