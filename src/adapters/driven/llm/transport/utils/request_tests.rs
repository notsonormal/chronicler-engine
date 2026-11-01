//! The seam contract of the LLM transport request side: one
//! `call_chat_completions` call sends one chat-completions request. The stub
//! endpoint shows what the transport puts on the wire.

use serde_json::{json, Value};

use crate::adapters::driven::llm::transport::{
    call_chat_completions, ChatCompletionRequest, ChatCompletionsEndpoint,
};
use crate::domain::model::generation_parameters::{CallSettings, ENGINE_KEYS};
use crate::domain::model::settings::ReasoningEffort;
use crate::test_support::chat_completions_mock::ChatCompletionsMock;

/// A call that sets no sampler and no custom parameter.
fn plain_call() -> CallSettings {
    CallSettings {
        max_tokens: 1024,
        ..CallSettings::default()
    }
}

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
    call: &'a CallSettings,
    system_prompt: &'a str,
    user_text: &'a str,
    reasoning_effort: Option<ReasoningEffort>,
) -> ChatCompletionRequest<'a> {
    ChatCompletionRequest {
        model: "test-model",
        system_prompt,
        user_text,
        call,
        session_id: None,
        reasoning_effort,
        provider_routing: None,
    }
}

// CASE transport-empty-system-prompt-omits-system-message: an empty system prompt sends the user message alone
#[test]
fn transport_empty_system_prompt_omits_the_system_message() {
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request(&call, "", "user question", None),
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
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request(&call, "system instruction", "user question", None),
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
    let call = CallSettings {
        max_tokens: 512,
        ..plain_call()
    };
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "what is 2+2?", None),
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
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", Some(ReasoningEffort::High)),
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
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", None),
    )
    .unwrap();

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
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", None),
    )
    .unwrap();

    assert_eq!(
        endpoint
            .received_headers()
            .get("user-agent")
            .map(String::as_str),
        Some(concat!("chronicler-engine/", env!("CARGO_PKG_VERSION")))
    );
}

// CASE transport-samplers-sent: a temperature and a top_p go on the wire and into the recorded request
#[test]
fn transport_sends_the_temperature_and_the_top_p() {
    let call = CallSettings {
        temperature: Some(0.7),
        top_p: Some(0.9),
        ..plain_call()
    };
    let endpoint = ChatCompletionsMock::start();

    let result = call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", None),
    )
    .unwrap();

    let body = endpoint.received_body();
    assert_eq!(body["temperature"], 0.7);
    assert_eq!(body["top_p"], 0.9);
    let recorded: Value = serde_json::from_str(&result.raw_request_json).unwrap();
    assert_eq!(recorded["temperature"], 0.7);
    assert_eq!(recorded["top_p"], 0.9);
}

// CASE transport-unset-samplers-omitted: a request with no sampler and no custom parameter sends no `temperature`, no `top_p` and no other key than the engine keys
#[test]
fn transport_without_samplers_sends_only_the_engine_keys() {
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", None),
    )
    .unwrap();

    let body = endpoint.received_body();
    let mut keys: Vec<&str> = body
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    let mut engine_keys = ENGINE_KEYS;
    engine_keys.sort_unstable();
    assert_eq!(keys, engine_keys);
}

// CASE transport-custom-parameters-sent: the keys of the custom object go on the wire as they are
#[test]
fn transport_sends_the_custom_parameters() {
    let call = CallSettings {
        custom: json!({"top_k": 40, "min_p": 0.05, "stop": ["END"]})
            .as_object()
            .unwrap()
            .clone(),
        ..plain_call()
    };
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", None),
    )
    .unwrap();

    let body = endpoint.received_body();
    assert_eq!(body["top_k"], 40);
    assert_eq!(body["min_p"], 0.05);
    assert_eq!(body["stop"], json!(["END"]));
}

// CASE transport-engine-keys-win-over-custom: a custom key that the engine also writes does not change the engine value
#[test]
fn transport_engine_keys_win_over_the_custom_parameters() {
    let call = CallSettings {
        temperature: Some(0.3),
        custom: json!({
            "model": "other-model",
            "messages": [],
            "stream": true,
            "max_tokens": 1,
            "temperature": 2.0,
            "reasoning_effort": "low",
        })
        .as_object()
        .unwrap()
        .clone(),
        ..plain_call()
    };
    let endpoint = ChatCompletionsMock::start();

    call_chat_completions(
        &plain(&endpoint),
        &request(&call, "be helpful", "question", Some(ReasoningEffort::High)),
    )
    .unwrap();

    let body = endpoint.received_body();
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
    assert_eq!(body["stream"], false);
    assert_eq!(body["max_tokens"], 1024);
    assert_eq!(body["temperature"], 0.3);
    assert_eq!(body["reasoning_effort"], "high");
}

// CASE transport-session-id-sent: a session id goes on the wire and into the recorded request, and no session id sends no field
#[test]
fn transport_sends_the_session_id_only_when_it_has_one() {
    let call = plain_call();
    let endpoint = ChatCompletionsMock::start();
    let result = call_chat_completions(
        &plain(&endpoint),
        &ChatCompletionRequest {
            session_id: Some("a-session"),
            ..request(&call, "be helpful", "question", None)
        },
    )
    .unwrap();
    assert_eq!(endpoint.received_body()["session_id"], "a-session");
    assert!(
        result
            .raw_request_json
            .contains(r#""session_id":"a-session""#)
    );

    let without = ChatCompletionsMock::start();
    call_chat_completions(
        &plain(&without),
        &request(&call, "be helpful", "question", None),
    )
    .unwrap();
    assert!(without.received_body().get("session_id").is_none());
}
