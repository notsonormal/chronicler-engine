//! The seam contract of the chat-completions provider: a connection goes in
//! through `from_config`, one `complete` call goes out to a chat-completions
//! endpoint. The stub endpoint shows what the provider puts on the wire.

use crate::adapters::driven::llm::providers::chat_completions::ChatCompletionsBackend;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{LlmConnection, ReasoningEffort};
use crate::test_support::chat_completions_mock::{ChatCompletionsMock, STUB_ANSWER};
use crate::test_support::test_connection;

fn connection_to(
    endpoint: &ChatCompletionsMock,
    provider: LlmBackendType,
    model: &str,
) -> LlmConnection {
    LlmConnection {
        model: model.into(),
        api_key: Some("sk-test".into()),
        base_url: Some(endpoint.base_url.clone()),
        ..test_connection("stub-connection", provider)
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

// CASE chat-completions-answers: a call returns the completion text and names the provider, the connection's model and the calling agent
#[test]
fn chat_completions_complete_answers_with_the_completion_and_its_names() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::OpenRouter,
        "gpt-4o",
    ));

    let result = backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(result.text, STUB_ANSWER);
    assert_eq!(result.backend_name, "OpenRouter");
    assert_eq!(result.model_name, "gpt-4o");
    assert_eq!(result.agent_name, "narrator");
    assert_eq!(endpoint.received_body()["model"], "gpt-4o");
}

// CASE chat-completions-ollama-named: an Ollama connection names Ollama as the provider
#[test]
fn chat_completions_complete_names_ollama_for_an_ollama_connection() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::Ollama,
        "llama3",
    ));

    let result = backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(result.backend_name, "Ollama");
}

// CASE chat-completions-openrouter-key-and-title: an OpenRouter connection sends its key and the app title
#[test]
fn chat_completions_openrouter_sends_the_key_and_the_app_title() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::OpenRouter,
        "gpt-4o",
    ));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    let headers = endpoint.received_headers();
    assert_eq!(
        headers.get("authorization").map(String::as_str),
        Some("Bearer sk-test")
    );
    assert_eq!(
        headers.get("x-title").map(String::as_str),
        Some("Chronicler Engine")
    );
}

// CASE chat-completions-ollama-no-key-no-title: an Ollama connection sends no key and no app title
#[test]
fn chat_completions_ollama_sends_no_key_and_no_app_title() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::Ollama,
        "llama3",
    ));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    let headers = endpoint.received_headers();
    assert!(!headers.contains_key("authorization"), "{headers:?}");
    assert!(!headers.contains_key("x-title"), "{headers:?}");
}

// CASE chat-completions-reasoning-effort-passed: the connection's reasoning effort goes on the wire
#[test]
fn chat_completions_complete_sends_the_reasoning_effort_of_the_connection() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&LlmConnection {
        reasoning_effort: Some(ReasoningEffort::Low),
        ..connection_to(&endpoint, LlmBackendType::Ollama, "llama3")
    });

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(endpoint.received_body()["reasoning_effort"], "low");
}

// CASE chat-completions-single-user-message: with single user message on, every provider sends one user message that starts with the system prompt, and no system message
#[test]
fn chat_completions_single_user_message_sends_no_system_message() {
    for provider in LlmBackendType::SELECTABLE {
        let endpoint = ChatCompletionsMock::start();
        let backend = ChatCompletionsBackend::from_config(&LlmConnection {
            single_user_message: true,
            ..connection_to(&endpoint, provider, "llama3")
        });

        backend
            .complete("narrator", "system rules", "user text", None)
            .unwrap();

        let body = endpoint.received_body();
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1, "{provider:?}: {messages:?}");
        assert_eq!(messages[0]["role"], "user", "{provider:?}");
        assert_eq!(
            messages[0]["content"], "[SYSTEM]\nsystem rules\n\nuser text",
            "{provider:?}"
        );
    }
}

// CASE chat-completions-gemma4-closes-thinking-channel: a Gemma 4 model on Ollama gets the turn and thought markers after the user text
#[test]
fn chat_completions_ollama_closes_the_thinking_channel_of_a_gemma4_model() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::Ollama,
        "gemma4:latest",
    ));

    backend
        .complete("narrator", "system", "User prompt", None)
        .unwrap();

    assert_eq!(
        sent_user_content(&endpoint),
        "User prompt\n<|turn>model\n<|channel>thought\n<channel|>"
    );
}

// CASE chat-completions-other-models-unchanged: any other Ollama model gets the user text as given
#[test]
fn chat_completions_ollama_sends_the_user_text_of_another_model_unchanged() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::Ollama,
        "llama3",
    ));

    backend
        .complete("narrator", "system", "User prompt", None)
        .unwrap();

    assert_eq!(sent_user_content(&endpoint), "User prompt");
}

// CASE chat-completions-gemma4-marker-ollama-only: a Gemma 4 model on another provider gets the user text as given
#[test]
fn chat_completions_gemma4_marker_is_for_ollama_only() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::OpenRouter,
        "google/gemma-4-26b",
    ));

    backend
        .complete("narrator", "system", "User prompt", None)
        .unwrap();

    assert_eq!(sent_user_content(&endpoint), "User prompt");
}

// CASE chat-completions-opencode-go-session-header: an OpenCode Go connection sends its key, and its connection id as the session header
#[test]
fn chat_completions_opencode_go_sends_the_connection_id_as_the_session() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&LlmConnection {
        id: "conn-1793000000000-a1b2c3d4".into(),
        ..connection_to(&endpoint, LlmBackendType::OpenCodeGo, "deepseek-v4.1-flash")
    });

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    let headers = endpoint.received_headers();
    assert_eq!(
        headers.get("x-opencode-session").map(String::as_str),
        Some("conn-1793000000000-a1b2c3d4")
    );
    assert_eq!(
        headers.get("authorization").map(String::as_str),
        Some("Bearer sk-test")
    );
}

// CASE chat-completions-no-session-header: an OpenRouter connection sends no session header
#[test]
fn chat_completions_openrouter_sends_no_session_header() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&connection_to(
        &endpoint,
        LlmBackendType::OpenRouter,
        "gpt-4o",
    ));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert!(
        !endpoint
            .received_headers()
            .contains_key("x-opencode-session")
    );
}

// CASE chat-completions-unsaved-connection-session: a connection with no id, as the form test sends, still sends a session value
#[test]
fn chat_completions_opencode_go_names_the_session_of_an_unsaved_connection() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&LlmConnection {
        id: String::new(),
        ..connection_to(&endpoint, LlmBackendType::OpenCodeGo, "deepseek-v4.1-flash")
    });

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(
        endpoint
            .received_headers()
            .get("x-opencode-session")
            .map(String::as_str),
        Some("unsaved-connection")
    );
}

fn routed(
    endpoint: &ChatCompletionsMock,
    provider: LlmBackendType,
    provider_order: &[&str],
    pin_provider_order: bool,
) -> LlmConnection {
    LlmConnection {
        provider_order: provider_order.iter().map(|&host| host.into()).collect(),
        pin_provider_order,
        ..connection_to(endpoint, provider, "deepseek/deepseek-v4-flash")
    }
}

// CASE chat-completions-routing-order-sent: an OpenRouter connection with a provider order sends that order and allows fallbacks
#[test]
fn chat_completions_openrouter_sends_the_provider_order() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&routed(
        &endpoint,
        LlmBackendType::OpenRouter,
        &["deepinfra", "novita"],
        false,
    ));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(
        endpoint.received_body()["provider"],
        serde_json::json!({"order": ["deepinfra", "novita"], "allow_fallbacks": true})
    );
}

// CASE chat-completions-routing-pinned: a pinned provider order sends no fallbacks
#[test]
fn chat_completions_pinned_provider_order_allows_no_fallbacks() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&routed(
        &endpoint,
        LlmBackendType::OpenRouter,
        &["deepinfra"],
        true,
    ));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert_eq!(
        endpoint.received_body()["provider"],
        serde_json::json!({"order": ["deepinfra"], "allow_fallbacks": false})
    );
}

// CASE chat-completions-routing-no-order: a connection with no provider order sends no provider field, even when pinned
#[test]
fn chat_completions_without_a_provider_order_sends_no_provider_field() {
    let endpoint = ChatCompletionsMock::start();
    let backend = ChatCompletionsBackend::from_config(&routed(
        &endpoint,
        LlmBackendType::OpenRouter,
        &[],
        true,
    ));

    backend
        .complete("narrator", "system", "user", None)
        .unwrap();

    assert!(endpoint.received_body().get("provider").is_none());
}

// CASE chat-completions-routing-openrouter-only: a provider order on a provider without routing sends no provider field
#[test]
fn chat_completions_provider_order_is_for_openrouter_only() {
    for provider in [LlmBackendType::Ollama, LlmBackendType::OpenCodeGo] {
        let endpoint = ChatCompletionsMock::start();
        let backend = ChatCompletionsBackend::from_config(&routed(
            &endpoint,
            provider,
            &["deepinfra"],
            false,
        ));

        backend
            .complete("narrator", "system", "user", None)
            .unwrap();

        assert!(
            endpoint.received_body().get("provider").is_none(),
            "{provider:?}"
        );
    }
}
