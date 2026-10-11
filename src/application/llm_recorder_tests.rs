//! Unit tests for `LlmCallRecorder` orchestrator.
//!
//! Tests verify the orchestration logic: provider gets called, sanitization
//! runs, forensics are persisted, errors propagate.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::llm_recorder::SaveLlmMessageFn;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::error::{EngineError, LlmFailure};
use crate::test_support::{make_noop_save_fn, make_test_recorder_with_storage};

struct BodyFailureProvider {
    failure: fn() -> LlmFailure,
}

impl LlmProvider for BodyFailureProvider {
    fn model(&self) -> &str {
        "deepseek-v4-flash"
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }

    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm((self.failure)()))
    }
}

const EMPTY_BODY: &str = r#"{"choices":[{"message":{"content":null,"reasoning":"let me think"},"finish_reason":"stop"}]}"#;
const SPENT_BODY: &str = r#"{"choices":[{"message":{"content":null},"finish_reason":"length"}]}"#;

const _: fn() = || {
    fn assert<T: Send + Sync>() {}
    assert::<LlmCallRecorder>();
};

#[test]
fn complete_happy_path_calls_provider_and_persists_forensics() {
    let provider = Arc::new(MockBackend::new());
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    let result = recorder
        .complete("narrator", "system prompt", "user prompt", None)
        .expect("complete should succeed");

    // Provider was called and returned text
    assert!(!result.text.is_empty());

    // Forensics persisted exactly one message
    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error")
        .pop()
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
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    let result = recorder
        .complete("narrator", "system", "user", None)
        .expect("complete should succeed");

    // Sanitized result text should NOT contain the thought tag
    assert!(!result.text.contains("<thought>"));
    assert!(result.text.contains("Hello, user!"));

    // Raw response JSON in forensics IS the original (unsanitized)
    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error")
        .pop()
        .expect("message should be saved");
    assert!(saved.raw_response_json.contains("<thought>"));

    // Parsed response in forensics IS sanitized
    assert!(!saved.parsed_response.contains("<thought>"));
    assert!(saved.parsed_response.contains("Hello, user!"));
}

#[test]
fn complete_records_the_raw_body_of_an_empty_response() {
    let raw_body = EMPTY_BODY;
    let provider = Arc::new(BodyFailureProvider {
        failure: || LlmFailure::EmptyResponse {
            raw_response: EMPTY_BODY.to_string(),
        },
    });
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    let err = recorder
        .complete("narrator", "system", "user", None)
        .expect_err("an empty content is a failure, never an answer");
    assert!(matches!(
        err,
        EngineError::Llm(LlmFailure::EmptyResponse { .. })
    ));

    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error")
        .pop()
        .expect("a failed attempt must be recorded");

    assert_eq!(saved.backend_name, "OpenRouter");
    assert_eq!(saved.model_name, "deepseek-v4-flash");
    assert_eq!(
        saved.raw_response_json, raw_body,
        "the stored row must keep the provider body"
    );
    assert!(
        saved.raw_request_json.is_empty(),
        "the transport returns the request payload only on success"
    );
    assert!(
        saved.parsed_response.is_empty(),
        "reasoning text must never become the parsed answer"
    );
    assert_eq!(
        saved.error_message.as_deref(),
        Some("LLM Error: empty response")
    );
}

#[test]
fn complete_records_a_spent_token_budget_for_role_health() {
    let provider = Arc::new(BodyFailureProvider {
        failure: || LlmFailure::TokenBudgetSpent {
            raw_response: SPENT_BODY.to_string(),
            completion_tokens: Some(2048),
            reasoning_tokens: Some(2041),
        },
    });
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    recorder
        .complete("quantifier", "system", "user", None)
        .expect_err("a spent budget with no content is a failure");

    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error")
        .pop()
        .expect("a failed attempt must be recorded");

    assert_eq!(saved.raw_response_json, SPENT_BODY);
    assert_eq!(
        saved.error_message.as_deref(),
        Some(
            "LLM Error: the model spent its whole token budget before writing an answer (2048 completion tokens, 2041 of them reasoning)"
        ),
        "role health and the banner read this column"
    );
}

#[test]
fn complete_propagates_provider_error_and_records_failure_row() {
    let provider = Arc::new(MockBackend::new().with_fail());
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    let err = recorder
        .complete("narrator", "system", "user", None)
        .unwrap_err();

    // Error propagates (MockBackend uses Narrative error variant)
    assert!(matches!(err, EngineError::Narrative(_)));

    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error")
        .pop()
        .expect("a failed attempt must be recorded");

    assert_eq!(saved.agent_name, "narrator");
    assert_eq!(saved.backend_name, "Mock");
    assert_eq!(saved.model_name, "mock");
    assert_eq!(saved.system_prompt, "system");
    assert_eq!(saved.user_prompt, "user");
    let text = saved.error_message.expect("the failure marker must be set");
    assert!(text.starts_with("LLM Error:"), "{text}");
    assert!(text.contains("configured_failure"), "{text}");

    // No response arrived, so every response column is empty.
    assert!(saved.raw_request_json.is_empty());
    assert!(saved.raw_response_json.is_empty());
    assert!(saved.parsed_response.is_empty());
}

#[test]
fn complete_failed_then_succeeded_leaves_both_rows() {
    let provider = Arc::new(MockBackend::new().with_fail_first_n(1));
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    recorder
        .complete("narrator", "s", "u", None)
        .expect_err("the first attempt fails");
    recorder
        .complete("narrator", "s", "u", None)
        .expect("the second attempt succeeds");

    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error");
    assert_eq!(saved.len(), 2, "each attempt leaves its own row");
    assert!(
        saved[0].error_message.is_some(),
        "the first row is the failure"
    );
    assert!(
        saved[1].error_message.is_none(),
        "the second row is the success"
    );
}

#[test]
fn complete_success_row_has_no_failure_marker() {
    // Boundary: a transport success is a success row even when the caller
    // cannot parse the response (e.g. a 200 with no parseable options).
    let provider = Arc::new(
        MockBackend::new().with_prompt_responses(vec!["no parseable options here".to_string()]),
    );
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(provider, Arc::clone(&storage));

    recorder
        .complete("options", "s", "u", None)
        .expect("the transport succeeded");

    let saved = storage
        .list_latest_llm_messages(10)
        .expect("list should not error");
    assert_eq!(saved.len(), 1);
    assert!(
        saved[0].error_message.is_none(),
        "an unparseable 200 response is still a success row"
    );
}

#[test]
fn complete_returns_provider_error_even_when_failure_save_fails() {
    // The failure row is diagnostics; a failure to write it must not mask the
    // provider error the caller retries or falls back on.
    let provider = Arc::new(MockBackend::new().with_fail());
    let save_fn: SaveLlmMessageFn = Arc::new(|_| Err(EngineError::Io("disk full".into())));
    let recorder = LlmCallRecorder::new(provider, save_fn);

    let err = recorder.complete("narrator", "s", "u", None).unwrap_err();
    assert!(
        matches!(err, EngineError::Narrative(_)),
        "the provider error must survive a failed forensics write"
    );
}

#[test]
fn complete_propagates_closure_save_error() {
    let provider = Arc::new(MockBackend::new());
    let save_fn: SaveLlmMessageFn = Arc::new(|_| Err(EngineError::Io("disk full".into())));
    let recorder = LlmCallRecorder::new(provider, save_fn);

    let err = recorder
        .complete("narrator", "system", "user", None)
        .unwrap_err();

    // Error propagates from the save-fn closure
    assert!(matches!(err, EngineError::Io(_)));
    assert!(err.to_string().contains("disk full"));
}

#[test]
fn provider_accessor_returns_injected_provider() {
    let original: Arc<dyn LlmProvider> = Arc::new(MockBackend::new());
    let recorder = LlmCallRecorder::new(original.clone(), make_noop_save_fn());

    assert!(Arc::ptr_eq(
        &original,
        &recorder.provider().expect("fixed recorder resolves")
    ));
}

#[test]
fn recorder_with_configurable_mock_backend() {
    // Verify that various MockBackend configurations work through the recorder
    let mocking_empty = Arc::new(MockBackend::new().with_empty_response());
    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(mocking_empty, Arc::clone(&storage));

    let result = recorder.complete("narrator", "sys", "user", None).unwrap();
    assert_eq!(result.text, "");
    assert_eq!(
        storage
            .list_latest_llm_messages(10)
            .expect("list should not error")
            .len(),
        1
    );
}

#[test]
fn resolver_is_consulted_on_every_call() {
    // Production recorders resolve per call so a dashboard connection edit takes
    // effect without a restart; assert the resolver runs again rather than caching.
    use std::sync::atomic::{AtomicUsize, Ordering};

    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let resolve: crate::application::llm_recorder::ProviderResolver = Arc::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(MockBackend::new()) as Arc<dyn LlmProvider>)
    });
    let recorder = LlmCallRecorder::with_resolver(resolve, make_noop_save_fn());

    recorder.complete("narrator", "s", "u", None).unwrap();
    recorder.complete("narrator", "s", "u", None).unwrap();

    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "the resolver must run once per complete(), not once at construction"
    );
}

#[test]
fn resolver_error_fails_the_call_instead_of_falling_back() {
    let resolve: crate::application::llm_recorder::ProviderResolver = Arc::new(|| {
        Err(EngineError::Config(
            "narration_connection_id 'gone' is not in the connections list".into(),
        ))
    });
    let recorder = LlmCallRecorder::with_resolver(resolve, make_noop_save_fn());

    let err = recorder
        .complete("narrator", "s", "u", None)
        .expect_err("an unresolvable provider must error, never serve a fallback");
    assert!(err.to_string().contains("gone"));
}
