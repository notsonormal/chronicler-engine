//! HTTP E2E tests for switching swipes (POST /message/:id/swipe/:index).

use axum::http::StatusCode;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::test_support::ScriptedModel;

use crate::support::app_wiring::app_with_script;
use crate::support::http_fixtures::OFFERED_SET;
use crate::support::http_requests::{fetch_body, post_action, post_empty, wait_settled};

/// The first swipe switch the rendered log offers, as `(message id, target
/// index)`.
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

async fn last_message_with_two_swipes(app: &axum::Router, state: &AppState) -> (u64, usize) {
    assert!(post_action(app, "look").await.status().is_success());
    assert!(wait_settled(state, 1000).await, "the turn should complete");

    assert!(post_empty(app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(state, 1000).await, "the retry should complete");

    let log = fetch_body(app, "/fragment/story-log").await;
    first_switch_target(&log)
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.1
#[tokio::test]
async fn test_switch_swipe_leaves_the_game_idle_http() {
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .turn("You look around again."),
    );

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
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .options(OFFERED_SET)
            .turn("You look around.")
            .options(OFFERED_SET)
            .turn("You look around again.")
            .turn("You look around once more.")
            .options(OFFERED_SET),
    );

    // Turn 2's pre-turn snapshot carries the live option set, so the retry's
    // Swipe snapshot does too — the stale set the switch must not restore.
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 1000).await);
    assert!(post_action(&app, "/options").await.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let (message_id, target) = last_message_with_two_swipes(&app, &state).await;

    assert!(post_action(&app, "/options").await.status().is_success());
    assert!(wait_settled(&state, 1000).await);
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
