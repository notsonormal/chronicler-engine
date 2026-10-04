//! HTTP E2E tests for switching swipes (POST /message/:id/swipe/:index).

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::agents::options::OptionsAgent;
use chronicler_engine::application::agents::quantifier::QuantifierAgent;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::settings::AppSettings;

use crate::support::app_wiring::app_with_narrator_and_registry;
use crate::support::http_requests::{fetch_body, post_action, post_empty, wait_idle};

/// App wired with an options agent so the dock can be populated, and a
/// narrator that returns a non-empty narration for any prompt.
fn options_app() -> (axum::Router, AppState, Arc<Storage>) {
    let mut registry = AgentRegistry::default();
    // A High-confidence quantifier result suppresses the uncertain-NPC
    // System fallback message, so the Narration stays the last Message and
    // keeps its swipe controls.
    registry.add_agent(Box::new(QuantifierAgent::with_provider(
        "quantifier".to_string(),
        Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": []}"#.to_string()]),
        ) as Arc<dyn LlmProvider>,
    )));
    registry.add_agent(Box::new(OptionsAgent::with_provider(
        "options".to_string(),
        Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>,
    )));
    app_with_narrator_and_registry(
        Arc::new(MockBackend::default()),
        registry,
        AppSettings::default(),
    )
}

/// The first swipe switch the rendered log offers, as `(message id, target
/// index)`. Reading the switch out of the fragment keeps the test on the HTTP
/// seam instead of reaching into storage.
fn first_switch_target(log_fragment: &str) -> (u64, usize) {
    let marker = "switchSwipe(";
    let start = log_fragment
        .find(marker)
        .expect("the log must render an enabled swipe switch")
        + marker.len();
    let rest = &log_fragment[start..];
    let end = rest.find(')').expect("the switch call must close");
    let mut args = rest[..end].split(',').map(str::trim);
    let id = args
        .next()
        .expect("switch call has an id")
        .parse::<u64>()
        .expect("switch call id is numeric");
    let index = args
        .next()
        .expect("switch call has an index")
        .parse::<usize>()
        .expect("switch call index is numeric");
    (id, index)
}

/// Drive a turn, then a second swipe of it, and return the switch target.
async fn last_message_with_two_swipes(app: &axum::Router, state: &AppState) -> (u64, usize) {
    assert!(post_action(app, "look").await.status().is_success());
    assert!(wait_idle(state, 1000).await, "the turn should complete");

    assert!(post_empty(app, "/swipe/new").await.status().is_success());
    assert!(wait_idle(state, 1000).await, "the retry should complete");

    let log = fetch_body(app, "/fragment/story-log").await;
    first_switch_target(&log)
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.1
#[tokio::test]
async fn test_switch_swipe_leaves_the_game_idle_http() {
    let (app, state, _storage) = options_app();

    let (message_id, target) = last_message_with_two_swipes(&app, &state).await;

    let resp = post_empty(&app, &format!("/message/{message_id}/swipe/{target}")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let status = fetch_body(&app, "/status/generating").await;
    assert_eq!(
        status.trim(),
        "idle",
        "a restored swipe must leave the game idle, not stuck generating"
    );
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.2
#[tokio::test]
async fn test_switch_swipe_drops_the_offered_option_set_http() {
    let (app, state, _storage) = options_app();

    // Turn 1, then a live option set. Turn 2's pre-turn snapshot then carries
    // that set, so the retry's Swipe snapshot does too — the stale set the
    // switch must not restore.
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    assert!(post_action(&app, "/options").await.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let (message_id, target) = last_message_with_two_swipes(&app, &state).await;

    assert!(post_action(&app, "/options").await.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    let before = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        before.contains("options-strip"),
        "the dock must show a set before the switch: {before}"
    );

    let resp = post_empty(&app, &format!("/message/{message_id}/swipe/{target}")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the restored swipe carries no option set, so the dock must not: {dock}"
    );
}
