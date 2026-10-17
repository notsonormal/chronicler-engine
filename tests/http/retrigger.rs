//! HTTP E2E tests for the retrigger endpoint (POST /retrigger).

use std::sync::Arc;

use axum::http::StatusCode;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, ModelRole, ScriptedModel, TestAppBuilder,
    TestStoredTriggerContext,
};

use crate::support::http_assertions::assert_story_log_ends_with_message;
use crate::support::http_fixtures::OFFERED_SET;
use crate::support::http_requests::{
    fetch_body, fetch_generating_status, post_action, post_empty, wait_settled,
};

/// Whether the story log holds an event Narration.
fn story_log_holds_event_narration(state: &AppState) -> bool {
    state
        .message_service
        .load_messages(active_game_of(state))
        .expect("the story log reads back")
        .iter()
        .any(|message| message.event_header().is_some())
}

// [docs/specs/retrigger.md] SCENARIO: 13.1
#[tokio::test]
async fn test_retrigger_creates_new_event_narration_no_rollback_http() {
    let model = ScriptedModel::new()
        .event_narration("The event fires again.")
        .analysis(analysis_report(&["npc_1"], None));
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", MessageType::Narration)
        .model(Arc::new(model))
        .build_with_state();

    let messages_before = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let n = messages_before.len();

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retrigger should complete"
    );

    let messages_after = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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
    // The event's reconciliation reads this answer; its movement destination
    // would move the player if a retrigger ran a movement-applying pass.
    let model = ScriptedModel::new()
        .event_narration("The event fires again.")
        .analysis(analysis_report(&[], Some("room2")));
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", MessageType::Narration)
        .model(Arc::new(model))
        .build_with_state();

    let room_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
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
    let app = TestAppBuilder::default_app();

    let req = axum::http::Request::builder()
        .uri("/retrigger")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retrigger with no trigger context should return bad request"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.2
#[tokio::test]
async fn test_retrigger_no_messages_returns_400_http() {
    let app = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
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
        "Retrigger with no messages should return bad request"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.3
#[tokio::test]
async fn test_retrigger_last_message_not_narration_returns_400_http() {
    let app = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("test input", MessageType::Input)
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
        "Retrigger when last message is Input should return bad request"
    );
}

// [docs/specs/retrigger.md] SCENARIO: 14.4
#[tokio::test]
async fn test_retrigger_last_message_is_event_continuation_returns_400_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", MessageType::Narration)
        .storage(Arc::clone(&storage))
        .build_with_state();

    let event_msg = Message::new(
        "Event narration",
        MessageType::Narration,
        None,
        Some("Event".to_string()),
    );
    let id = storage
        .insert_message(active_game_of(&state), &event_msg)
        .unwrap();
    if let Some(swipe) = event_msg.swipes.first() {
        let _ = storage.insert_swipe(id, swipe, 0);
    }

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
    // The event narration fails, so the event's reconciliation never runs.
    let model = ScriptedModel::new().failed(ModelRole::Event);
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", MessageType::Narration)
        .model(Arc::new(model))
        .build_with_state();

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retrigger should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("Trigger narration failed"),
        "error should mention 'Trigger narration failed', got: {status}"
    );
    let messages_after = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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
            m.message_type == MessageType::System && m.text().contains("Trigger narration failed")
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
        .storage(Arc::new(Storage::new_in_memory()))
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
    // The held Options refresh is the busy state retrigger meets, and it leaves
    // the story log's last Message the Narration the route validates.
    let model = Arc::new(ScriptedModel::new().options(OFFERED_SET));
    let hold = model.hold(ModelRole::Options);
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", MessageType::Narration)
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
    assert!(
        body_str.contains("Still thinking..."),
        "expected 'Still thinking...' in body, got: {body_str}"
    );

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
