use std::sync::Arc;

use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::state::LogType;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context_with_sqlite;

use crate::pipeline_helpers::{
    add_input_and_save, create_test_state_with_map, latest_state, save_state,
    wait_for_generation_complete,
};

#[test]
fn test_sequential_execute_retry_execute() {
    // Flow: Action A → Retry A → Action B → Verify history order and state consistency
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    // Step 1: Action A
    add_input_and_save(&ctx, "examine room");
    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action A should complete"
    );

    // Step 2: Retry A
    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Retry A should complete"
    );

    // Step 3: Action B
    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action B should complete"
    );

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "Should have exactly 2 input entries");

    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Narration)
        .collect();
    assert!(
        narrations.len() >= 2,
        "Should have narrations from both actions"
    );

    // Verify LLM calls were logged to SQLite storage
    let messages = ctx.storage.list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_sequential_execute_delete_execute() {
    // Flow: Action A → Delete A's narration → Action B → Verify clean state
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    // Step 1: Action A
    add_input_and_save(&ctx, "examine room");
    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action A should complete"
    );

    let guard = latest_state(&ctx);
    let narration_id = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.log_type == LogType::Narration)
        .map(|e| e.id)
        .expect("Should have a narration entry");

    // Step 2: Delete the narration
    {
        let mut state = latest_state(&ctx);
        state.narrative.history.retain(|m| m.id != narration_id);
        save_state(&ctx, &state);
    }

    // Step 3: Action B
    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());
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
    // Flow: async action A → async action B → retry
    // Verify sequential async actions and retry work correctly.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    // Step 1: Async action A
    add_input_and_save(&ctx, "hello");
    service.execute_action(ctx.clone(), "hello".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Step 2: Async action B
    add_input_and_save(&ctx, "examine room");
    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Step 3: Retry
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "Should have 2 input entries");
}

#[test]
fn test_three_actions_in_sequence() {
    // Flow: Action A → Action B → Action C
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    for action in ["examine room", "look around", "check inventory"] {
        add_input_and_save(&ctx, action);
        service.execute_action(ctx.clone(), action.to_string(), "Player".to_string());
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
        .filter(|e| e.log_type == LogType::Input)
        .collect();
    assert_eq!(inputs.len(), 3, "Should have 3 input entries");

    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Narration)
        .collect();
    assert!(
        narrations.len() >= 2,
        "Should have at least 2 narrations (look is sync, others async)"
    );
}

#[test]
fn test_delete_input_then_retry_fails_gracefully() {
    // Flow: Execute → delete input → Retry
    // Retry should find no input and fail gracefully.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "examine room");

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Delete the input entry
    {
        let mut state = latest_state(&ctx);
        state.narrative.history.clear();
        save_state(&ctx, &state);
    }

    // Retry should not panic or hang
    service.retry_last_response(ctx.clone());
    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry with no input should not leave state generating"
    );
}

#[test]
fn test_reset_clears_history_and_state() {
    // Flow: Execute action with movement → Reset → verify back to initial state
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk to room2");

    let quantifier = Arc::new(MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    service.execute_action(
        ctx.clone(),
        "walk to room2".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));
    let guard = latest_state(&ctx);
    assert_eq!(guard.movement.current_room_id, "room2");
    assert!(!guard.narrative.history().is_empty());

    // Reset: create fresh initial state
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
    // Flow: Execute → Reset → Execute again
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    // First action
    add_input_and_save(&ctx, "examine room");
    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Reset
    let fresh_state = create_test_state_with_map();
    save_state(&ctx, &fresh_state);

    // Second action after reset
    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Action after reset should complete"
    );

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Input)
        .collect();
    assert_eq!(
        inputs.len(),
        1,
        "After reset, only the second action's input should exist"
    );
}

#[test]
fn test_delete_mid_sequence() {
    // Flow: Action A → Action B → delete B's narration → Action C
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    // Action A
    add_input_and_save(&ctx, "examine room");
    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Action B
    add_input_and_save(&ctx, "look around");
    service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let narration_b_id = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.log_type == LogType::Narration)
        .map(|e| e.id)
        .expect("Should have narration B");

    // Delete B's narration
    {
        let mut state = latest_state(&ctx);
        state.narrative.history.retain(|m| m.id != narration_b_id);
        save_state(&ctx, &state);
    }

    // Action C
    add_input_and_save(&ctx, "check door");
    service.execute_action(ctx.clone(), "check door".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let inputs: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Input)
        .collect();
    assert_eq!(inputs.len(), 3, "Should have 3 input entries");

    let has_b_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.id == narration_b_id);
    assert!(!has_b_narration, "Deleted narration B should not reappear");
}
