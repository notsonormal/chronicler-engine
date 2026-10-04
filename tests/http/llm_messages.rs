//! HTTP E2E tests for the LLM Messages panel: a failed attempt is recorded and its failure text renders.

use std::sync::Arc;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::domain::model::settings::AppSettings;

use crate::support::app_wiring::app_with_narrator_and_registry;
use crate::support::http_requests::{fetch_body, post_action, wait_idle};

/// App on the default test data whose narrator recorder persists every
/// attempt into the shared storage.
fn narrator_app(narrator: Arc<MockBackend>) -> (axum::Router, AppState, Arc<Storage>) {
    app_with_narrator_and_registry(narrator, AgentRegistry::default(), AppSettings::default())
}

// [docs/specs/llm_messages.md] SCENARIO: 33.1
#[tokio::test]
async fn test_failed_narration_attempt_records_and_renders_failure_http() {
    let (app, state, _storage) = narrator_app(Arc::new(MockBackend::new().with_fail()));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");

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
    assert_eq!(failure.backend_name, "Mock");
    assert_eq!(failure.model_name, "mock");
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
        panel.contains("Mock / mock"),
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
    let (app, state, _storage) = narrator_app(Arc::new(MockBackend::new().with_fail_first_n(1)));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_idle(&state, 1000).await,
        "first action should complete"
    );

    let resp = post_action(&app, "listen").await;
    assert!(resp.status().is_success());
    assert!(
        wait_idle(&state, 1000).await,
        "second action should complete"
    );

    let messages = state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed");
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
