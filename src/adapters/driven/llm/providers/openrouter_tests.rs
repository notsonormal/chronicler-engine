//! The seam contract of the OpenRouter provider: a connection goes in through
//! `from_config`, one `complete` call goes out to a chat-completions endpoint.
//! The stub endpoint shows what the provider puts on the wire.

use crate::adapters::driven::llm::providers::openrouter::OpenRouterBackend;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{LlmProviderConfig, ReasoningEffort};
use crate::test_support::chat_completions_mock::{ChatCompletionsMock, STUB_ANSWER};
use crate::test_support::test_connection;

fn connection_to(endpoint: &ChatCompletionsMock) -> LlmProviderConfig {
    LlmProviderConfig {
        model: "gpt-4o".into(),
        api_key: Some("sk-test".into()),
        base_url: Some(endpoint.base_url.clone()),
        ..test_connection("or-stub", LlmBackendType::OpenRouter)
    }
}

// CASE openrouter-complete-answers: a call reaches the connection's base URL and returns the completion text, naming the provider, the connection's model and the calling agent
#[test]
fn openrouter_complete_answers_with_the_completion_and_its_names() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OpenRouterBackend::from_config(&connection_to(&endpoint));

    let result = backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(result.text, STUB_ANSWER);
    assert_eq!(result.backend_name, "OpenRouter");
    assert_eq!(result.model_name, "gpt-4o");
    assert_eq!(result.agent_name, "narrator");
    assert_eq!(endpoint.received_body()["model"], "gpt-4o");
}

// CASE openrouter-reasoning-effort-sent: the connection's reasoning effort goes on the wire
#[test]
fn openrouter_complete_sends_the_reasoning_effort_of_the_connection() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OpenRouterBackend::from_config(&LlmProviderConfig {
        reasoning_effort: Some(ReasoningEffort::Low),
        ..connection_to(&endpoint)
    });

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(
        endpoint.received_body()["reasoning"],
        serde_json::json!({"effort": "low"})
    );
}

// CASE openrouter-reasoning-effort-absent: a connection without a reasoning effort sends no reasoning field
#[test]
fn openrouter_complete_without_a_reasoning_effort_sends_no_reasoning_field() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OpenRouterBackend::from_config(&connection_to(&endpoint));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert!(endpoint.received_body().get("reasoning").is_none());
}
