//! HTTP E2E tests for the retrigger endpoint (POST /retrigger).

use std::sync::Arc;

use axum::http::StatusCode;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    make_test_pipeline_with_mock_quantifier, make_test_recorder, TestAppBuilder,
    TestStoredTriggerContext,
};

use crate::test_helpers::{post_empty, wait_idle};

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 13.1
#[tokio::test]
async fn test_retrigger_creates_new_event_narration_no_rollback_http() {
    let narrator = Arc::new(MockBackend::default());
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>,
    );
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", None, MessageType::Narration)
        .pipeline(pipeline)
        .build_with_state();

    let messages_before = state.message_service.load_messages().unwrap();
    let n = messages_before.len();

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retrigger should complete");

    let messages_after = state.message_service.load_messages().unwrap();
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
    assert!(matches!(
        state
            .message_service
            .load_or_fresh()
            .narrative
            .input_buffer
            .status,
        GenerationStatus::Idle
    ));
}

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 13.2
#[tokio::test]
async fn test_retrigger_does_not_rerun_quantifier_http() {
    let narrator = Arc::new(MockBackend::default());
    // Quantifier WOULD move the player if called — retrigger must NOT call it.
    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
            .to_string(),
    ])) as Arc<dyn LlmProvider>;
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        quantifier,
    );
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", None, MessageType::Narration)
        .pipeline(pipeline)
        .build_with_state();

    let room_before = state
        .message_service
        .load_or_fresh()
        .movement
        .current_room_id
        .clone();

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retrigger should complete");

    let room_after = state
        .message_service
        .load_or_fresh()
        .movement
        .current_room_id;
    assert_eq!(
        room_before, room_after,
        "retrigger must not re-run quantifier (room unchanged)"
    );
}

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 14.1
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

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 14.2
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

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 14.3
#[tokio::test]
async fn test_retrigger_last_message_not_narration_returns_400_http() {
    let app = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("test input", Some("Player"), MessageType::Input)
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

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 14.4
#[tokio::test]
async fn test_retrigger_last_message_is_event_continuation_returns_400_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", None, MessageType::Narration)
        .storage(Arc::clone(&storage))
        .build();

    let event_msg = Message::new(
        None,
        "Event narration",
        MessageType::Narration,
        None,
        Some("Event".to_string()),
    );
    let id = storage.insert_message(&event_msg).unwrap();
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

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 14.5
#[tokio::test]
async fn test_retrigger_trigger_narration_failure_sets_error_http() {
    let narrator = Arc::new(MockBackend::default().with_trigger_narration_fail());
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>,
    );
    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", None, MessageType::Narration)
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_idle(&state, 1000).await,
        "retrigger should complete (with error)"
    );

    let gs = state.message_service.load_or_fresh();
    match &gs.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("Trigger narration failed"),
                "error should mention 'Trigger narration failed', got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    let messages_after = state.message_service.load_messages().unwrap();
    // A System message is logged on trigger failure (expected). Assert no new
    // event Narration was added — the failed continuation is not persisted.
    let new_event_narrations = messages_after
        .iter()
        .filter(|m| m.message_type == MessageType::Narration && m.event_header().is_some())
        .count();
    assert_eq!(
        new_event_narrations, 0,
        "failed retrigger should not persist a new event narration"
    );
    assert!(
        messages_after.iter().any(|m| {
            m.message_type == MessageType::System && m.text().contains("Trigger narration failed")
        }),
        "System log should mention trigger failure"
    );
}

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 14.6
#[tokio::test]
async fn test_retrigger_no_game_context_returns_400_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let game_id = storage.current_game_id();
    storage.delete_game(game_id).unwrap();

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

// [chronicler_engine/docs/specs/retrigger.md] SCENARIO: 15.1
#[tokio::test]
async fn test_retrigger_concurrent_generation_returns_still_thinking_http() {
    use chronicler_engine::application::errors::ProcessActionResult;

    let (app, state) = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("Main narration", None, MessageType::Narration)
        .build_with_state();

    let game_id = state.game_catalogue.current_game_id();
    let mut game_state = state.message_service.load_or_fresh();
    let (_, _, claim) = state
        .generation_gate
        .try_claim(game_id, &mut game_state, state.message_service.as_ref())
        .expect("pre-claim should succeed");
    assert!(matches!(claim, ProcessActionResult::Started));

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
}
