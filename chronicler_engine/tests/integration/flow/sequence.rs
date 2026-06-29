use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::test_support::make_test_context_with_sqlite;

use crate::pipeline_helpers::{
    add_input_and_save, create_test_state_with_map, latest_state, save_state,
    wait_for_generation_complete,
};

#[test]
fn test_sequential_execute_retry_execute() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    add_input_and_save(&ctx, "examine room");
    service.execute_action(ctx.clone(), "examine room".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action A should complete"
    );

    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Retry A should complete"
    );

    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action B should complete"
    );

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "Should have exactly 2 input entries");

    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.len() >= 2,
        "Should have narrations from both actions"
    );

    let messages = ctx.storage.list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_sequential_execute_delete_execute() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    add_input_and_save(&ctx, "examine room");
    service.execute_action(ctx.clone(), "examine room".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action A should complete"
    );

    let guard = latest_state(&ctx);
    let narration_id = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.id)
        .expect("Should have a narration entry");

    {
        let mut state = latest_state(&ctx);
        state.narrative.history.retain(|m| m.id != narration_id);
        save_state(&ctx, &state);
    }

    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action B should complete"
    );

    let guard = latest_state(&ctx);
    let has_deleted_narration = guard
        .narrative
        .history()
        .iter()
        .any(|e| e.id == narration_id);
    assert!(
        !has_deleted_narration,
        "Deleted narration should not reappear"
    );
}

#[test]
fn test_async_action_sequence_then_retry() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    add_input_and_save(&ctx, "hello");
    service.execute_action(ctx.clone(), "hello".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    add_input_and_save(&ctx, "examine room");
    service.execute_action(ctx.clone(), "examine room".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "Should have 2 input entries");
}

#[test]
fn test_three_actions_in_sequence() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    for action in ["examine room", "look around", "check inventory"] {
        add_input_and_save(&ctx, action);
        service.execute_action(ctx.clone(), action.to_string());
        assert!(
            wait_for_generation_complete(&ctx, 1000),
            "Action '{action}' should complete"
        );
    }

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 3, "Should have 3 input entries");

    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.len() >= 2,
        "Should have at least 2 narrations (look is sync, others async)"
    );
}

#[test]
fn test_delete_input_then_retry_fails_gracefully() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "examine room");

    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(ctx.clone(), "examine room".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    {
        let mut state = latest_state(&ctx);
        state.narrative.history.clear();
        save_state(&ctx, &state);
    }

    service.retry_last_response(ctx.clone());
    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry with no input should not leave state generating"
    );
}

#[test]
fn test_reset_clears_history_and_state() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk to room2");

    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ]));

    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    service.execute_action(ctx.clone(), "walk to room2".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let guard = latest_state(&ctx);
    assert_eq!(guard.movement.current_room_id, "room2");
    assert!(!guard.narrative.history().is_empty());

    let fresh_state = create_test_state_with_map();
    save_state(&ctx, &fresh_state);

    let guard = latest_state(&ctx);
    assert_eq!(
        guard.movement.current_room_id, "room1",
        "After reset: back to room1"
    );
    assert!(
        guard.narrative.history().is_empty(),
        "After reset: history cleared"
    );
}

#[test]
fn test_reset_then_execute_works() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    add_input_and_save(&ctx, "examine room");
    service.execute_action(ctx.clone(), "examine room".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let fresh_state = create_test_state_with_map();
    save_state(&ctx, &fresh_state);

    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action after reset should complete"
    );

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Input)
        .collect();
    assert_eq!(
        inputs.len(),
        1,
        "After reset, only the second action's input should exist"
    );
}

#[test]
fn test_delete_mid_sequence() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    add_input_and_save(&ctx, "examine room");
    service.execute_action(ctx.clone(), "examine room".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let narration_b_id = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.id)
        .expect("Should have narration B");

    {
        let mut state = latest_state(&ctx);
        state.narrative.history.retain(|m| m.id != narration_b_id);
        save_state(&ctx, &state);
    }

    add_input_and_save(&ctx, "check door");
    service.execute_action(ctx.clone(), "check door".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 3, "Should have 3 input entries");

    let has_b_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.id == narration_b_id);
    assert!(!has_b_narration, "Deleted narration B should not reappear");
}
