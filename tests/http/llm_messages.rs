//! HTTP E2E tests for the LLM Messages panel: a failed attempt is recorded and its failure text renders.

use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::test_support::{analysis_report, ScriptedModel};

use crate::support::app_wiring::app_with_script;
use crate::support::http_requests::{fetch_body, post_action, wait_settled};

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
    assert!(
        !panel.contains("Raw Request JSON") && !panel.contains("Raw Response JSON"),
        "an entry with no response must hide its empty Raw JSON sections: {panel}"
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
    assert!(
        panel.contains("listen"),
        "the second narration must render: {panel}"
    );
}

// [docs/specs/llm_messages.md] SCENARIO: 33.3
#[tokio::test]
async fn test_llm_messages_fragment_renders_collapsed_cards_with_their_view_state_kept_http() {
    let (app, state, _storage) = app_with_script(ScriptedModel::new().failed(Role::Narrator));
    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");
    let id = state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed")
        .first()
        .expect("the attempt is recorded")
        .id;

    let panel = fetch_body(&app, "/fragment/llm-messages").await;

    assert!(
        !panel.contains("llm-messages-panel"),
        "the fragment is the content of the panel, and repeats no llm-messages-panel element: {panel}"
    );
    assert!(
        panel.contains(&format!(r#"id="llm-msg-{id}""#)),
        "the card must carry its stable id: {panel}"
    );
    assert!(
        panel.contains(r#"aria-expanded="false" data-keep="aria-expanded""#),
        "the header must report collapsed and list aria-expanded in data-keep: {panel}"
    );
    assert!(
        panel.contains(&format!(
            r#"id="llm-msg-body-{id}" hidden data-keep="hidden""#
        )),
        "the body must be hidden and list hidden in data-keep: {panel}"
    );
    assert!(
        !panel.contains("<details>")
            && !panel.contains(r#"<details class="llm-message-prompt-details">"#),
        "each disclosure must list its open state in data-keep: {panel}"
    );
}
