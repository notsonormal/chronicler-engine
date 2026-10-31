//! The seam contract of the LLM transport request side: one
//! `call_chat_completions` call sends one chat-completions request. The stub
//! endpoint shows what the transport puts on the wire.

use crate::adapters::driven::llm::transport::{
    call_chat_completions, ChatCompletionRequest, ChatCompletionsEndpoint,
};
use crate::domain::model::settings::ReasoningEffort;
use crate::test_support::chat_completions_mock::ChatCompletionsMock;

/// The mock's address, with no key, no title and no session.
fn plain(mock: &ChatCompletionsMock) -> ChatCompletionsEndpoint<'_> {
    ChatCompletionsEndpoint {
        base_url: &mock.base_url,
        api_key: None,
        title: None,
        session: None,
    }
}

fn request<'a>(
    system_prompt: &'a str,
    user_text: &'a str,
    reasoning_effort: Option<ReasoningEffort>,
) -> ChatCompletionRequest<'a> {
    ChatCompletionRequest {
        model: "test-model",
        system_prompt,
        user_text,
        max_tokens: Some(1024),
        reasoning_effort,
        provider_routing: None,
    }
}

// CASE transport-empty-system-prompt-omits-system-message: an empty system prompt sends the user message alone
#[test]
fn transport_empty_system_prompt_omits_the_system_message() {
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(&plain(&endpoint), &request("", "user question", None)).unwrap();

    let body = endpoint.received_body();
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1, "only the user message");
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "user question");
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["stream"], false);
}

// CASE transport-system-prompt-sent-before-user: a non-empty system prompt sends the system message, then the user message
#[test]
fn transport_system_prompt_goes_before_the_user_message() {
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request("system instruction", "user question", None),
    )
    .unwrap();

    let body = endpoint.received_body();
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[0]["content"], "system instruction");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "user question");
}

// CASE transport-max-tokens-integer: the token limit goes on the wire and into the recorded request as a JSON integer
#[test]
fn transport_sends_max_tokens_as_an_integer() {
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &plain(&endpoint),
        &ChatCompletionRequest {
            max_tokens: Some(512),
            ..request("be helpful", "what is 2+2?", None)
        },
    )
    .unwrap();

    assert_eq!(endpoint.received_body()["max_tokens"], 512);
    let recorded_request: serde_json::Value =
        serde_json::from_str(&result.raw_request_json).unwrap();
    assert_eq!(recorded_request["max_tokens"], 512);
}

// CASE transport-reasoning-effort-sent: a reasoning effort goes on the wire and into the recorded request as the flat `reasoning_effort` field
#[test]
fn transport_sends_the_reasoning_effort() {
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &plain(&endpoint),
        &request("be helpful", "question", Some(ReasoningEffort::High)),
    )
    .unwrap();

    let body = endpoint.received_body();
    assert_eq!(body["reasoning_effort"], "high");
    assert!(
        body.get("reasoning").is_none(),
        "no nested reasoning object"
    );
    assert!(
        result
            .raw_request_json
            .contains(r#""reasoning_effort":"high""#),
        "{}",
        result.raw_request_json
    );
}

// CASE transport-no-reasoning-effort-omits-field: no reasoning effort sends neither a `reasoning_effort` nor a `reasoning` field
#[test]
fn transport_without_a_reasoning_effort_sends_no_reasoning_field() {
    let endpoint = ChatCompletionsMock::start();

    let result =
        call_chat_completions(&plain(&endpoint), &request("be helpful", "question", None)).unwrap();

    let body = endpoint.received_body();
    assert!(body.get("reasoning_effort").is_none());
    assert!(body.get("reasoning").is_none());
    assert!(
        !result.raw_request_json.contains("reasoning"),
        "{}",
        result.raw_request_json
    );
}

// CASE transport-user-agent-names-the-engine: every request names the engine and its version in the `User-Agent` header
#[test]
fn transport_user_agent_names_the_engine_and_its_version() {
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(&plain(&endpoint), &request("be helpful", "question", None)).unwrap();

    assert_eq!(
        endpoint
            .received_headers()
            .get("user-agent")
            .map(String::as_str),
        Some(concat!("chronicler-engine/", env!("CARGO_PKG_VERSION")))
    );
}
