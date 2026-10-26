//! HTTP E2E tests for the LLM Messages panel: a failed attempt is recorded and its failure text renders, the list sends a header for each attempt, and the record of one attempt opens in the dialog.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration, Utc};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::llm_message::LlmMessage;
use chronicler_engine::test_support::{analysis_report, ScriptedModel};

use crate::support::app_wiring::app_with_script;
use crate::support::http_requests::{fetch_body, post_action, response_body, wait_settled};

// [docs/specs/llm_messages.md] SCENARIO: 33.1
#[tokio::test]
async fn test_failed_narration_attempt_records_and_renders_failure_http() {
    let (app, state, _storage) = app_with_script(ScriptedModel::new().failed(Role::Narrator));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed");
    let failures: Vec<_> = messages
        .iter()
        .filter(|m| m.error_message.is_some())
        .collect();
    assert_eq!(
        failures.len(),
        1,
        "the failed attempt must be recorded once"
    );
    let failure = failures[0];
    assert_eq!(failure.agent_name, "narrator");
    assert_eq!(failure.backend_name, "Scripted");
    assert_eq!(failure.model_name, "scripted");
    assert!(
        failure
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("configured_failure"),
        "the failure text must be stored verbatim: {:?}",
        failure.error_message
    );
    assert!(failure.raw_request_json.is_empty());
    assert!(failure.raw_response_json.is_empty());
    assert!(failure.parsed_response.is_empty());

    let panel = fetch_body(&app, "/fragment/llm-messages").await;
    assert!(
        panel.contains("narrator"),
        "the Agent must be named: {panel}"
    );
    assert!(
        panel.contains("Scripted / scripted"),
        "the backend/model must be named: {panel}"
    );
    assert!(
        panel.contains("configured_failure"),
        "the failure text must render: {panel}"
    );

    let record = fetch_body(&app, &format!("/fragment/llm-messages/{}", failure.id)).await;
    assert!(
        record.contains("configured_failure"),
        "the record must carry the failure text: {record}"
    );
    assert!(
        !record.contains("Raw Request JSON") && !record.contains("Raw Response JSON"),
        "an entry with no response must hide its empty Raw JSON sections: {record}"
    );
}

// [docs/specs/llm_messages.md] SCENARIO: 33.2
#[tokio::test]
async fn test_failed_attempt_survives_a_later_success_http() {
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .failed(Role::Narrator)
            .narration("You listen.")
            .after_narration(analysis_report(&[], None)),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "first action should complete"
    );

    let resp = post_action(&app, "listen").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "second action should complete"
    );

    let messages: Vec<_> = state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed")
        .into_iter()
        .filter(|m| m.agent_name == "narrator")
        .collect();
    assert_eq!(messages.len(), 2, "each attempt leaves its own row");
    assert_eq!(
        messages
            .iter()
            .filter(|m| m.error_message.is_some())
            .count(),
        1,
        "the first attempt stays a failure row"
    );
    assert_eq!(
        messages
            .iter()
            .filter(|m| m.error_message.is_none())
            .count(),
        1,
        "the second attempt is a success row"
    );

    let panel = fetch_body(&app, "/fragment/llm-messages").await;
    assert!(
        panel.contains("configured_failure"),
        "the failure text must stay visible: {panel}"
    );
    let success = messages
        .iter()
        .find(|m| m.error_message.is_none())
        .expect("the second attempt is a success row");
    let record = fetch_body(&app, &format!("/fragment/llm-messages/{}", success.id)).await;
    assert!(
        record.contains("You listen."),
        "the record of the second attempt must carry the second narration: {record}"
    );
}

const SYSTEM_MARKER: &str = "SYSTEM-PROMPT-MARKER";
const USER_MARKER: &str = "USER-PROMPT-MARKER";
const RESPONSE_MARKER: &str = "PARSED-RESPONSE-MARKER";
const RAW_REQUEST_MARKER: &str = "RAW-REQUEST-MARKER";
const RAW_RESPONSE_MARKER: &str = "RAW-RESPONSE-MARKER";
const FAILURE_MARKER: &str = "FAILURE-TEXT-MARKER";

/// A recorded attempt that carries every field, so a marker shows where the
/// field reaches the client.
fn full_attempt(offset_secs: i64) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: "narrator".to_string(),
        backend_name: "Mock".to_string(),
        model_name: "mock-model".to_string(),
        system_prompt: SYSTEM_MARKER.to_string(),
        user_prompt: USER_MARKER.to_string(),
        raw_request_json: format!(r#"{{"request":"{RAW_REQUEST_MARKER}"}}"#),
        raw_response_json: format!(r#"{{"response":"{RAW_RESPONSE_MARKER}"}}"#),
        parsed_response: RESPONSE_MARKER.to_string(),
        error_message: None,
        created_at: Utc::now() + Duration::seconds(offset_secs),
    }
}

fn failed_attempt(offset_secs: i64) -> LlmMessage {
    LlmMessage {
        parsed_response: String::new(),
        raw_response_json: String::new(),
        error_message: Some(FAILURE_MARKER.to_string()),
        ..full_attempt(offset_secs)
    }
}

/// The ids of the recorded attempts, oldest first.
fn recorded_ids(state: &chronicler_engine::adapters::driving::http::AppState) -> Vec<i64> {
    state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed")
        .iter()
        .map(|message| message.id)
        .collect()
}

async fn get(app: &axum::Router, uri: &str) -> axum::response::Response<Body> {
    app.clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn seed(storage: &Storage, messages: &[LlmMessage]) {
    for message in messages {
        storage
            .save_llm_message(message)
            .expect("seed an LLM message");
    }
}

// [docs/specs/llm_messages.md] SCENARIO: 33.3
#[tokio::test]
async fn test_llm_messages_list_sends_a_header_for_each_attempt_http() {
    let (app, state, storage) = app_with_script(ScriptedModel::new());
    seed(&storage, &[full_attempt(-10), failed_attempt(-5)]);
    let ids = recorded_ids(&state);
    assert_eq!(ids.len(), 2);

    let panel = fetch_body(&app, "/fragment/llm-messages").await;

    assert!(
        !panel.contains("llm-messages-panel"),
        "the fragment is the content of the panel, and repeats no llm-messages-panel element: {panel}"
    );
    for id in &ids {
        assert!(
            panel.contains(&format!(r#"id="llm-msg-{id}""#)),
            "the card must carry its stable id: {panel}"
        );
        assert!(
            panel.contains(&format!(
                r##"hx-get="/fragment/llm-messages/{id}" hx-target="#edit-dialog-body" hx-swap="innerHTML""##
            )),
            "the Open control must load the record into the dialog: {panel}"
        );
    }
    assert!(
        panel.contains("narrator") && panel.contains("Mock / mock-model"),
        "the header names the Agent and the backend/model: {panel}"
    );
    assert!(
        panel.contains("llm-message-error"),
        "the failed attempt carries the error mark: {panel}"
    );
    for text in [
        SYSTEM_MARKER,
        USER_MARKER,
        RESPONSE_MARKER,
        RAW_REQUEST_MARKER,
        RAW_RESPONSE_MARKER,
        "Raw Request JSON",
        "<details",
    ] {
        assert!(
            !panel.contains(text),
            "the list sends no record text ({text}): {panel}"
        );
    }
}

// [docs/specs/llm_messages.md] SCENARIO: 33.4
#[tokio::test]
async fn test_llm_message_record_answers_the_full_record_as_a_dialog_http() {
    let (app, state, storage) = app_with_script(ScriptedModel::new());
    seed(&storage, &[full_attempt(-10), failed_attempt(-5)]);
    let ids = recorded_ids(&state);

    let response = get(&app, &format!("/fragment/llm-messages/{}", ids[0])).await;
    assert_eq!(response.status(), StatusCode::OK);
    let record = response_body(response).await;

    assert!(
        record.contains("data-dialog-form") && record.contains(r#"<h2 id="edit-dialog-title">"#),
        "the record is a dialog fragment with the title that names the dialog: {record}"
    );
    assert!(
        record.contains("data-close-dialog"),
        "Close closes the dialog: {record}"
    );
    for text in [
        SYSTEM_MARKER,
        USER_MARKER,
        RESPONSE_MARKER,
        RAW_REQUEST_MARKER,
        RAW_RESPONSE_MARKER,
        "Mock / mock-model",
    ] {
        assert!(
            record.contains(text),
            "the record carries the whole attempt ({text}): {record}"
        );
    }
    assert!(
        !record.contains("<form") && !record.contains("<input") && !record.contains("<textarea"),
        "the record is read-only: {record}"
    );

    let failed = get(&app, &format!("/fragment/llm-messages/{}", ids[1])).await;
    assert_eq!(failed.status(), StatusCode::OK);
    let failed = response_body(failed).await;
    assert!(
        failed.contains(FAILURE_MARKER) && !failed.contains(RESPONSE_MARKER),
        "the record of a failed attempt carries its failure text and no response: {failed}"
    );
}

// [docs/specs/llm_messages.md] SCENARIO: 33.5
#[tokio::test]
async fn test_llm_message_record_of_a_missing_id_is_a_404_http() {
    let (app, state, storage) = app_with_script(ScriptedModel::new());
    seed(&storage, &[full_attempt(-10)]);
    let missing = recorded_ids(&state)[0] + 1000;

    let response = get(&app, &format!("/fragment/llm-messages/{missing}")).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response_body(response).await;
    assert_eq!(
        body,
        r#"<div class="error-message">LLM message not found</div>"#
    );
}
