//! HTTP E2E tests for the retrigger endpoint (POST /retrigger).

use std::sync::Arc;

use axum::http::StatusCode;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, empty_sqlite_storage, ModelRole, ScriptedModel,
    TestAppBuilder, TestMap, TestNpc,
};

use crate::support::app_wiring::app_with_a_triggered_npc;
use crate::support::http_assertions::{assert_still_thinking, assert_story_log_ends_with_message};
use crate::support::http_fixtures::{story_of, OFFERED_SET};
use crate::support::http_requests::{
    fetch_body, fetch_generating_status, post_action, post_empty, wait_settled,
};

fn story_log_holds_event_narration(state: &AppState) -> bool {
    story_of(state)
        .iter()
        .any(|message| message.event_header().is_some())
}

// [docs/specs/retrigger.md] SCENARIO: 13.1
#[tokio::test]
async fn test_retrigger_creates_new_event_narration_no_rollback_http() {
    // The Action's Trigger fires but its continuation fails, so the main Narration
    // keeps the trigger context that makes Retrigger available.
    let npc = TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0);
    let model = ScriptedModel::new()
        .narration("Main narration")
        .analysis(analysis_report(&["npc_1"], None))
        .failed(ModelRole::Event)
        .event_narration("The event fires again.")
        .analysis(analysis_report(&["npc_1"], None));
    let (app, state) = app_with_a_triggered_npc(TestMap::single_room("room_1"), npc, model);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should settle after its failed continuation"
    );
    let n = story_of(&state).len();

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retrigger should complete"
    );

    let messages_after = story_of(&state);
    assert_eq!(
        messages_after.len(),
        n + 1,
        "retrigger appends exactly one new message (no rollback)"
    );
    let new_msg = messages_after.last().expect("new message");
    assert_eq!(new_msg.message_type, MessageType::Narration);
    assert!(
        new_msg.event_header().is_some(),
        "new message should have event_header"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "retrigger should finish in the Idle state"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 13.2
#[tokio::test]
async fn test_retrigger_does_not_move_the_player_http() {
    // The Action moves into "room2"; the event's reconciliation then names
    // "room_1" as a tripwire, because a retrigger applies no movement.
    let npc = TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0);
    let model = ScriptedModel::new()
        .narration("You walk into the next room.")
        .analysis(analysis_report(&["npc_1"], Some("room2")))
        .failed(ModelRole::Event)
        .event_narration("The event fires again.")
        .analysis(analysis_report(&[], Some("room_1")));
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the moving Action should settle after its failed continuation"
    );
    let room_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(room_before, "room2", "the Action moved the player");

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retrigger should complete"
    );

    let room_after = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(
        room_before, room_after,
        "retrigger must not re-run quantifier (room unchanged)"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.1
#[tokio::test]
async fn test_retrigger_no_trigger_context_returns_400_http() {
    // A fresh Game's opening Narration holds no trigger context.
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let before = story_of(&state);

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "Retrigger with no trigger context should return bad request"
    );

    assert!(
        !story_log_holds_event_narration(&state),
        "no retrigger was started: its event Narration never lands"
    );
    assert_eq!(
        story_of(&state).len(),
        before.len(),
        "the refused Retrigger saves no Message"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.3
#[tokio::test]
async fn test_retrigger_last_message_not_narration_returns_400_http() {
    // The Action's own Narration is deleted, so the story ends on its typed Input.
    let model = ScriptedModel::new().turn("You look around.");
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(model))
        .build_with_state();

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );
    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK, "the delete should accept");

    let req = axum::http::Request::builder()
        .uri("/retrigger")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retrigger when last message is Input should return bad request"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.4
#[tokio::test]
async fn test_retrigger_last_message_is_event_continuation_returns_400_http() {
    // The Action fires its Trigger and the event Narration lands, so the story ends
    // on an event Narration.
    let npc = TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0);
    let model = ScriptedModel::new()
        .narration("Main narration")
        .analysis(analysis_report(&["npc_1"], None))
        .event_narration("The event lands.")
        .analysis(analysis_report(&["npc_1"], None));
    let (app, state) = app_with_a_triggered_npc(TestMap::single_room("room_1"), npc, model);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );
    assert!(
        story_log_holds_event_narration(&state),
        "the Action's Trigger fires, so an event Narration lands"
    );

    let req = axum::http::Request::builder()
        .uri("/retrigger")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retrigger when last message is event continuation should return bad request"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.5
#[tokio::test]
async fn test_retrigger_trigger_narration_failure_sets_error_http() {
    // The Action's Trigger fires but its continuation fails, leaving the trigger
    // context; the retrigger's continuation then fails too.
    let npc = TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0);
    let model = ScriptedModel::new()
        .narration("Main narration")
        .analysis(analysis_report(&["npc_1"], None))
        .failed(ModelRole::Event)
        .failed(ModelRole::Event);
    let (app, state) = app_with_a_triggered_npc(TestMap::single_room("room_1"), npc, model);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should settle after its failed continuation"
    );

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retrigger should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("The event narration failed."),
        "the failure line of a failed event narration, got: {status}"
    );
    let messages_after = story_of(&state);
    let new_event_narrations = messages_after
        .iter()
        .filter(|m| m.message_type == MessageType::Narration && m.event_header().is_some())
        .count();
    assert_eq!(
        new_event_narrations, 0,
        "failed retrigger should not persist a new event narration"
    );
    assert!(
        !messages_after.iter().any(|m| {
            m.message_type == MessageType::System
                && m.text().contains("The event narration failed.")
        }),
        "the engine error must not be written into the story log: {messages_after:?}"
    );
    let last = messages_after
        .last()
        .expect("the main narration survives the failed retrigger");
    assert_eq!(
        last.message_type,
        MessageType::Narration,
        "the main narration must stay the last entry: {messages_after:?}"
    );

    let story_log = fetch_body(&app, "/fragment/story-log").await;
    assert!(
        story_log.contains("retrigger-btn"),
        "the failed trigger must leave Retrigger available: {story_log}"
    );
    assert_story_log_ends_with_message(
        &story_log,
        last.id,
        "the main narration must be the last rendered entry",
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.6
#[tokio::test]
async fn test_retrigger_no_game_context_returns_400_http() {
    // No Game in storage and no active-Game pointer: the `CurrentGame`
    // extractor answers the 400 before the handler runs.
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(empty_sqlite_storage()))
        .skip_seeding(true)
        .build();

    let req = axum::http::Request::builder()
        .uri("/retrigger")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should fail when game context is missing"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 15.1
#[tokio::test]
async fn test_retrigger_concurrent_generation_returns_still_thinking_http() {
    // The held Options refresh is the busy state retrigger meets.
    let model = Arc::new(ScriptedModel::new().options(OFFERED_SET));
    let hold = model.hold(ModelRole::Options);
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::clone(&model))
        .build_with_state();

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success(), "the options Job should accept");
    assert!(
        hold.wait_started(1000),
        "the held options call should reach the model"
    );

    let req = axum::http::Request::builder()
        .uri("/retrigger")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .expect("read body");
    let body_str = String::from_utf8_lossy(&body);
    assert_still_thinking(&body_str);

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Job should complete once released"
    );
    assert!(
        !story_log_holds_event_narration(&state),
        "no retrigger was started: its event Narration never lands"
    );
}
