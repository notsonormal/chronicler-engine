//! HTTP E2E tests for the retry endpoint (POST /swipe/new).

use std::sync::Arc;

use axum::http::StatusCode;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
use chronicler_engine::test_support::{
    insert_message_with_swipe, make_test_pipeline_with_mock_quantifier, make_test_recorder,
    seed_event_flow, TestAppBuilder, TestDataBuilder, TestMap,
};

use crate::test_helpers::{app_with_narrator, post_action, post_empty, wait_idle};

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 9.1
#[tokio::test]
async fn test_retry_replaces_narration_with_new_swipe_http() {
    let narrator = Arc::new(
        MockBackend::default().with_narrations(vec!["First.".to_string(), "Second.".to_string()]),
    );
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry should complete");

    let messages = state.message_service.load_messages().unwrap();
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(narrations.len(), 1, "exactly one Narration message");
    assert_eq!(narrations[0].swipes.len(), 2, "retry appended a swipe");
    assert_eq!(narrations[0].text(), "Second.", "active swipe is the retry");
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

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 9.2
#[tokio::test]
async fn test_retry_reruns_quantifier_moves_player_http() {
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let narrator = Arc::new(MockBackend::default());
    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": []}"#.to_string(),
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
        .data(data)
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");
    {
        let gs = state.message_service.load_or_fresh();
        assert_eq!(gs.movement.current_room_id, "room_1", "first action: stay");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry should complete");
    let gs = state.message_service.load_or_fresh();
    assert_eq!(
        gs.movement.current_room_id, "room2",
        "retry re-ran quantifier → moved to room2"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 9.3
#[tokio::test]
async fn test_retry_preserves_input_message_http() {
    let (app, state) = app_with_narrator(Arc::new(MockBackend::default()));

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry should complete");

    let messages = state.message_service.load_messages().unwrap();
    let input = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message must exist");
    assert_eq!(input.text(), "walk around", "Input text preserved by retry");
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 9.4
#[tokio::test]
async fn test_retry_uses_edited_input_text_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let narrator = Arc::new(MockBackend::default());
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::clone(&storage),
        recorder,
        Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>,
    );
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let input_id = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message")
        .id;
    let index = storage.require_active_swipe_index(input_id).unwrap();
    storage
        .update_swipe_text(input_id, index, "sprint forward")
        .expect("update_swipe_text should succeed");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry should complete");

    let messages = state.message_service.load_messages().unwrap();
    let retry_narration = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .next_back()
        .expect("retry narration");
    assert!(
        retry_narration.text().contains("sprint forward"),
        "retry narration should reflect edited input, got: {}",
        retry_narration.text()
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 9.5
#[tokio::test]
async fn test_retry_reevaluates_triggers_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    let shopkeeper = TestNpc::with_room_scoped_trigger(
        "shopkeeper",
        "Shopkeeper Sarah",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .npcs(vec![shopkeeper])
        .build();
    // room_npcs defaults to ["npc_1"]; the shopkeeper is not in any room's
    // NPC list (their trigger is room-gated), so no trigger fires on action 1.

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "You walk around.".to_string(),
        "You enter room2.".to_string(),
        "The shopkeeper greets you.".to_string(),
    ]));
    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": []}"#.to_string(),
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
        .data(data)
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");
    {
        let gs = state.message_service.load_or_fresh();
        let events = gs
            .narrative
            .history()
            .iter()
            .filter(|m| m.event_header.is_some())
            .count();
        assert_eq!(events, 0, "first action: no trigger (player not in room2)");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry should complete");
    let gs = state.message_service.load_or_fresh();
    let events = gs
        .narrative
        .history()
        .iter()
        .filter(|m| m.event_header.is_some())
        .count();
    assert_eq!(
        events, 1,
        "retry moved player to room2 → trigger should fire"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 9.6
#[tokio::test]
async fn test_retry_completes_when_quantifier_returns_no_movement_http() {
    let narrator = Arc::new(MockBackend::default());
    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": []}"#.to_string(),
        r#"{"npcs_in_room": []}"#.to_string(),
    ])) as Arc<dyn LlmProvider>;
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        quantifier,
    );
    let (app, state) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_idle(&state, 1000).await,
        "retry should complete even with no movement"
    );
    let gs = state.message_service.load_or_fresh();
    assert!(
        !gs.narrative.input_buffer.status.is_generating(),
        "status should not be Generating"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 10.1
#[tokio::test]
async fn test_event_retry_replaces_event_narration_with_new_swipe_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let narrator = Arc::new(MockBackend::default());
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::clone(&storage),
        recorder,
        Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>,
    );
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .build_with_state();

    seed_event_flow(&state, &storage).unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "event retry should complete");

    let messages = state.message_service.load_messages().unwrap();
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(narrations.len(), 2, "main + event narration");
    let event_narration = narrations
        .iter()
        .find(|m| m.event_header().is_some())
        .expect("event narration with event_header");
    assert_eq!(
        event_narration.swipes.len(),
        2,
        "event retry appends exactly one swipe (original + new)"
    );
    assert_eq!(
        event_narration.active_swipe_index, 1,
        "new swipe should be the active swipe"
    );
    assert_ne!(
        event_narration.text(),
        "Event narration",
        "active swipe text should be the new retry narration, not the original"
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

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 10.2
#[tokio::test]
async fn test_event_retry_does_not_rerun_quantifier_http() {
    let storage = Arc::new(Storage::new_in_memory());
    // Quantifier would move the player if called; event retry must NOT call it.
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let narrator = Arc::new(MockBackend::default());
    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
            .to_string(),
    ])) as Arc<dyn LlmProvider>;
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline =
        make_test_pipeline_with_mock_quantifier(Arc::clone(&storage), recorder, quantifier);
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .build_with_state();

    seed_event_flow(&state, &storage).unwrap();

    let room_before = state
        .message_service
        .load_or_fresh()
        .movement
        .current_room_id
        .clone();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "event retry should complete");

    let room_after = state
        .message_service
        .load_or_fresh()
        .movement
        .current_room_id;
    assert_eq!(
        room_before, room_after,
        "event retry must not re-run quantifier (room unchanged)"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.1
#[tokio::test]
async fn test_retry_no_input_returns_400_http() {
    let app = TestAppBuilder::default_app();

    let req = axum::http::Request::builder()
        .uri("/swipe/new")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retry with no input should return bad request"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.2
#[tokio::test]
async fn test_retry_no_game_context_returns_400_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let game_id = storage.current_game_id();
    storage.delete_game(game_id).unwrap();

    let req = axum::http::Request::builder()
        .uri("/swipe/new")
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

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.3
#[tokio::test]
async fn test_retry_anchor_no_snapshot_returns_500_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    // Seed an Input message with snapshot_id = None (broken integrity).
    let input_msg = Message::new(
        Some("Player".to_string()),
        "look",
        MessageType::Input,
        None,
        None,
    );
    insert_message_with_swipe(&storage, &input_msg).unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "anchor with no snapshot_id should 500"
    );
    let gs = state.message_service.load_or_fresh();
    match &gs.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("snapshot_id") || msg.contains("snapshot"),
                "error should mention snapshot, got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.4
#[tokio::test]
async fn test_retry_anchor_snapshot_deleted_returns_500_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    // Seed an Input message with a snapshot_id that doesn't exist in storage.
    let mut input_msg = Message::new(
        Some("Player".to_string()),
        "look",
        MessageType::Input,
        None,
        None,
    );
    input_msg.set_snapshot_id(Some(999_999)); // non-existent snapshot
    insert_message_with_swipe(&storage, &input_msg).unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "anchor with deleted snapshot should 500"
    );
    let gs = state.message_service.load_or_fresh();
    match &gs.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("no snapshot found") || msg.contains("snapshot"),
                "error should mention snapshot not found, got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.5
#[tokio::test]
async fn test_retry_llm_failure_sets_error_http() {
    let narrator = Arc::new(MockBackend::default().with_fail());
    let (app, state) = app_with_narrator(narrator);

    // Seed Input + Narration directly; the with_fail() narrator fails on retry.
    {
        let mut gs = state.message_service.load_or_fresh();
        gs.add_message(
            "look".to_string(),
            Some("Player".to_string()),
            MessageType::Input,
        );
        gs.add_message(
            "Original narration.".to_string(),
            None,
            MessageType::Narration,
        );
        state
            .message_service
            .save_message_and_snapshot(&mut gs)
            .expect("seed input + narration");
    }

    let swipes_before = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_idle(&state, 1000).await,
        "retry should complete (with error)"
    );

    let gs = state.message_service.load_or_fresh();
    match &gs.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(!msg.is_empty(), "error message should be non-empty: {msg}");
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    let swipes_after = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);
    assert_eq!(
        swipes_before, swipes_after,
        "failed retry should not append a swipe"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.6
#[tokio::test]
async fn test_retry_empty_narration_sets_error_http() {
    let narrator = Arc::new(MockBackend::default().with_empty_response());
    let (app, state) = app_with_narrator(narrator);

    {
        let mut gs = state.message_service.load_or_fresh();
        gs.add_message(
            "look".to_string(),
            Some("Player".to_string()),
            MessageType::Input,
        );
        gs.add_message(
            "Original narration.".to_string(),
            None,
            MessageType::Narration,
        );
        state
            .message_service
            .save_message_and_snapshot(&mut gs)
            .expect("seed input + narration");
    }

    let swipes_before = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_idle(&state, 1000).await,
        "retry should complete (with error)"
    );

    let gs = state.message_service.load_or_fresh();
    match &gs.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("empty"),
                "error message should mention 'empty', got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    let swipes_after = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);
    assert_eq!(
        swipes_before, swipes_after,
        "failed retry should not append a swipe"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.7
#[tokio::test]
async fn test_retry_room_not_found_sets_error_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    // Seed an Input whose snapshot points at a non-existent room (retry anchor).
    let mut gs = state.message_service.load_or_fresh();
    gs.movement.current_room_id = "non_existent_room".to_string();
    gs.add_message(
        "look".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let snap = GameStateSnapshot::from_game_state(&gs);
    let input_snap_id = storage.save_snapshot(&snap).unwrap();
    if let Some(last) = gs.narrative.history.last_mut() {
        last.set_snapshot_id(Some(input_snap_id));
        let id = storage.insert_message(last).unwrap();
        if let Some(swipe) = last.swipes.first() {
            let _ = storage.insert_swipe(id, swipe, 0);
        }
    }

    // Seed a non-event Narration as the last message (spec 11.7 Given).
    let mut gs = state.message_service.load_or_fresh();
    gs.add_message("A calm scene.".to_string(), None, MessageType::Narration);
    let snap = GameStateSnapshot::from_game_state(&gs);
    let narration_snap_id = storage.save_snapshot(&snap).unwrap();
    if let Some(last) = gs.narrative.history.last_mut() {
        last.set_snapshot_id(Some(narration_snap_id));
        let id = storage.insert_message(last).unwrap();
        if let Some(swipe) = last.swipes.first() {
            let _ = storage.insert_swipe(id, swipe, 0);
        }
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_idle(&state, 1000).await,
        "retry should complete (with error)"
    );

    let gs = state.message_service.load_or_fresh();
    match &gs.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("Room not found"),
                "error should mention room invalid, got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }

    // Spec 11.7 Then: original narration's swipes unchanged (no new swipe appended).
    let messages = state.message_service.load_messages().unwrap();
    let narration = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration && m.event_header().is_none())
        .next_back()
        .expect("non-event narration preserved");
    assert_eq!(
        narration.swipes.len(),
        1,
        "original narration's swipes unchanged after room-not-found error"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 11.8
#[tokio::test]
async fn test_event_retry_trigger_narration_failure_sets_error_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let narrator = Arc::new(MockBackend::default().with_trigger_narration_fail());
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::clone(&storage),
        recorder,
        Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>,
    );
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .build_with_state();

    seed_event_flow(&state, &storage).unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_idle(&state, 1000).await,
        "event retry should complete (with error)"
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
    let messages = state.message_service.load_messages().unwrap();
    assert!(
        messages.iter().any(|m| {
            m.message_type == MessageType::Narration
                && m.event_header().is_none()
                && !m.text().is_empty()
        }),
        "main narration should be preserved"
    );
    assert!(
        messages.iter().any(|m| {
            m.message_type == MessageType::System && m.text().contains("Trigger narration failed")
        }),
        "System log should mention trigger failure"
    );
}

// [chronicler_engine/docs/specs/swipe_new.md] SCENARIO: 12.1
#[tokio::test]
async fn test_retry_concurrent_generation_returns_still_thinking_http() {
    use chronicler_engine::application::errors::ProcessActionResult;

    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let mut game_state = state.message_service.load_or_fresh();
    game_state.add_message(
        "test input".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    state
        .message_service
        .save_message_and_snapshot(&mut game_state)
        .expect("seed input message with snapshot");

    // Pre-claim the generation gate so retry sees it busy.
    let game_id = state.game_catalogue.current_game_id();
    let (_, _, claim) = state
        .generation_gate
        .try_claim(game_id, &mut game_state, state.message_service.as_ref())
        .expect("pre-claim should succeed");
    assert!(matches!(claim, ProcessActionResult::Started));

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 16384)
        .await
        .expect("read body");
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Still thinking..."),
        "expected 'Still thinking...' in body, got: {body_str}"
    );
}
