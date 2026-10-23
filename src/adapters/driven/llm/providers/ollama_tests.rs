//! The seam contract of the Ollama provider: a connection goes in through
//! `from_config`, one `complete` call goes out to a chat-completions endpoint.
//! The stub endpoint shows what the provider puts on the wire.

use crate::adapters::driven::llm::providers::ollama::OllamaBackend;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{LlmProviderConfig, ReasoningEffort};
use crate::test_support::chat_completions_mock::{ChatCompletionsMock, STUB_ANSWER};
use crate::test_support::test_connection;

fn connection_to(endpoint: &ChatCompletionsMock, model: &str) -> LlmProviderConfig {
    LlmProviderConfig {
        model: model.into(),
        base_url: Some(endpoint.base_url.clone()),
        ..test_connection("ollama-stub", LlmBackendType::Ollama)
    }
}

/// The content of the last message of the request the endpoint received.
fn sent_user_content(endpoint: &ChatCompletionsMock) -> String {
    let body = endpoint.received_body();
    let last = body["messages"]
        .as_array()
        .and_then(|messages| messages.last())
        .expect("the request carries messages");
    last["content"].as_str().expect("text content").to_string()
}

// CASE ollama-complete-answers: a call returns the completion text and names the provider, the connection's model and the calling agent
#[test]
fn ollama_complete_answers_with_the_completion_and_its_names() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OllamaBackend::from_config(&connection_to(&endpoint, "llama3"));

    let result = backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(result.text, STUB_ANSWER);
    assert_eq!(result.backend_name, "Ollama");
    assert_eq!(result.model_name, "llama3");
    assert_eq!(result.agent_name, "narrator");
    assert_eq!(endpoint.received_body()["model"], "llama3");
}

// CASE ollama-reasoning-effort-sent: the connection's reasoning effort goes on the wire and into the recorded request
#[test]
fn ollama_complete_sends_the_reasoning_effort_of_the_connection() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OllamaBackend::from_config(&LlmProviderConfig {
        reasoning_effort: Some(ReasoningEffort::Low),
        ..connection_to(&endpoint, "llama3")
    });

    let result = backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(
        endpoint.received_body()["reasoning"],
        serde_json::json!({"effort": "low"})
    );
    let recorded_request: serde_json::Value =
        serde_json::from_str(&result.raw_request_json).unwrap();
    assert_eq!(
        recorded_request["reasoning"],
        serde_json::json!({"effort": "low"})
    );
}

// CASE ollama-reasoning-effort-absent: a connection without a reasoning effort sends no reasoning field
#[test]
fn ollama_complete_without_a_reasoning_effort_sends_no_reasoning_field() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OllamaBackend::from_config(&connection_to(&endpoint, "llama3"));

    let result = backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert!(endpoint.received_body().get("reasoning").is_none());
    let recorded_request: serde_json::Value =
        serde_json::from_str(&result.raw_request_json).unwrap();
    assert!(recorded_request.get("reasoning").is_none());
}

// CASE ollama-gemma4-closes-thinking-channel: a Gemma 4 model gets the turn and thought markers after the user text
#[test]
fn ollama_complete_closes_the_thinking_channel_of_a_gemma4_model() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OllamaBackend::from_config(&connection_to(&endpoint, "gemma4:latest"));

    backend
        .complete("narrator", "system", "User prompt", None)
        .unwrap();

    let sent = sent_user_content(&endpoint);
    assert!(sent.starts_with("User prompt"));
    assert!(sent.contains("<|turn>model"));
    assert!(sent.contains("<|channel>thought"));
}

// CASE ollama-other-models-unchanged: any other model gets the user text as given
#[test]
fn ollama_complete_sends_the_user_text_of_another_model_unchanged() {
    let endpoint = ChatCompletionsMock::start();
    let backend = OllamaBackend::from_config(&connection_to(&endpoint, "llama3"));

    backend
        .complete("narrator", "system", "User prompt", None)
        .unwrap();

    assert_eq!(sent_user_content(&endpoint), "User prompt");
}
