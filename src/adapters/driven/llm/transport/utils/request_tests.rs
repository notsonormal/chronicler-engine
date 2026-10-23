//! The seam contract of the LLM transport request side: one
//! `call_chat_completions` call sends one chat-completions request. The stub
//! endpoint shows what the transport puts on the wire.

use crate::adapters::driven::llm::transport::{call_chat_completions, ChatCompletionRequest};
use crate::domain::model::settings::ReasoningEffort;
use crate::test_support::chat_completions_mock::ChatCompletionsMock;

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
    }
}

// CASE transport-empty-system-prompt-omits-system-message: an empty system prompt sends the user message alone
#[test]
fn transport_empty_system_prompt_omits_the_system_message() {
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &endpoint.base_url,
        None,
        None,
        &request("", "user question", None),
    )
    .unwrap();

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
        &endpoint.base_url,
        None,
        None,
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
        &endpoint.base_url,
        None,
        None,
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

// CASE transport-reasoning-effort-sent: a reasoning effort goes on the wire and into the recorded request
#[test]
fn transport_sends_the_reasoning_effort() {
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &endpoint.base_url,
        None,
        None,
        &request("be helpful", "question", Some(ReasoningEffort::High)),
    )
    .unwrap();

    assert_eq!(
        endpoint.received_body()["reasoning"],
        serde_json::json!({"effort": "high"})
    );
    assert!(
        result
            .raw_request_json
            .contains(r#""reasoning":{"effort":"high"}"#),
        "{}",
        result.raw_request_json
    );
}

// CASE transport-no-reasoning-effort-omits-field: no reasoning effort sends no reasoning field
#[test]
fn transport_without_a_reasoning_effort_sends_no_reasoning_field() {
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &endpoint.base_url,
        None,
        None,
        &request("be helpful", "question", None),
    )
    .unwrap();

    assert!(endpoint.received_body().get("reasoning").is_none());
    assert!(
        !result.raw_request_json.contains("reasoning"),
        "{}",
        result.raw_request_json
    );
}
