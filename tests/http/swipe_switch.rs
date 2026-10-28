//! HTTP E2E tests for switching swipes (POST /message/:id/swipe/:index).

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, ScriptedModel, TestAppBuilder, TestDataBuilder, TestMap,
};

use crate::support::app_wiring::app_with_script;
use crate::support::http_fixtures::{room_header_of, story_of, OFFERED_SET};
use crate::support::http_requests::{
    assert_answers_nothing, fetch_body, fetch_status_label, post_action, post_empty, response_body,
    wait_settled,
};

fn first_switch_target(log_fragment: &str) -> (u64, usize) {
    let marker = r#"hx-post="/message/"#;
    let start = log_fragment
        .find(marker)
        .expect("the log must render an enabled swipe switch")
        + marker.len();
    let rest = &log_fragment[start..];
    let end = rest.find('"').expect("the switch route must close");
    let mut parts = rest[..end].split("/swipe/");
    let id = parts
        .next()
        .expect("switch route has an id")
        .parse::<u64>()
        .expect("switch route id is numeric");
    let index = parts
        .next()
        .expect("switch route has an index")
        .parse::<usize>()
        .expect("switch route index is numeric");
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
    assert_answers_nothing(resp).await;

    let status = fetch_status_label(&app).await;
    assert_eq!(
        status, "Ready",
        "a restored swipe must leave the game idle, not stuck generating"
    );
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.2
#[tokio::test]
async fn test_switch_swipe_drops_the_offered_option_set_http() {
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(OFFERED_SET)
            .turn("You look around again.")
            .turn("You look around once more.")
            .options(OFFERED_SET),
    );

    // The `/options` runs leave the dock a live set, which the switch's own
    // write must clear: a switch writes the empty dock record.
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

// [docs/specs/swipe_switch.md] SCENARIO: 36.5
#[tokio::test]
async fn test_switch_swipe_past_the_last_swipe_is_refused_http() {
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .turn("You look around again."),
    );

    let (message_id, _target) = last_message_with_two_swipes(&app, &state).await;
    let before = story_of(&state)
        .into_iter()
        .find(|message| message.id == message_id)
        .expect("the Narration survives the refused switch");

    let resp = post_empty(&app, &format!("/message/{message_id}/swipe/2")).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = response_body(resp).await;
    assert!(
        body.contains("Swipe index out of bounds"),
        "the refusal names the index it could not reach: {body}"
    );

    let after = story_of(&state)
        .into_iter()
        .find(|message| message.id == message_id)
        .expect("the Narration survives the refused switch");
    assert_eq!(after.swipe_count(), 2, "the refused switch adds no Swipe");
    assert_eq!(
        after.active_swipe_index, before.active_swipe_index,
        "the refused switch leaves the active Swipe where it was"
    );
    assert_eq!(
        after.text(),
        before.text(),
        "the refused switch leaves the shown text as it was"
    );
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.4
#[tokio::test]
async fn test_switch_swipe_gives_back_the_state_of_that_swipe_http() {
    // The Action moves the player into "room2"; the Retry starts from the state
    // before it and stays put, so the two Swipes of the Narration hold the two
    // rooms.
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You walk north.")
                .after_narration(analysis_report(&[], Some("room2")))
                .narration("You stand still.")
                .after_narration(analysis_report(&[], None)),
        ))
        .build_with_state();

    let (message_id, target) = last_message_with_two_swipes(&app, &state).await;
    assert_eq!(target, 0, "the log offers the other Swipe of the pair");

    let room = |state: &AppState| {
        state
            .game_view_query
            .get_debug_state(active_game_of(state))
            .expect("get_debug_state should succeed")
            .current_room_id
    };

    assert_eq!(room(&state), "room_1", "the Retry stayed put");
    assert_eq!(room_header_of(&state, message_id), None);

    let resp = post_empty(&app, &format!("/message/{message_id}/swipe/{target}")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    assert_eq!(
        room(&state),
        "room2",
        "the first Swipe's state names the room its generation arrived in"
    );
    assert_eq!(
        room_header_of(&state, message_id).as_deref(),
        Some("Room room2"),
        "the Narration shows the room its first Swipe arrived in"
    );

    let resp = post_empty(&app, &format!("/message/{message_id}/swipe/1")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    assert_eq!(
        room(&state),
        "room_1",
        "the second Swipe's state comes back"
    );
    assert_eq!(room_header_of(&state, message_id), None);
}
