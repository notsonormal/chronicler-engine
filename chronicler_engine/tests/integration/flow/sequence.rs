//! Integration flow tests for action sequencing: execute→retry→execute, execute→delete→execute, async action ordering, and three-action sequence under realistic state churn.

use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::TestDataBuilder;

use chronicler_engine::application::action_pipeline::{execute_action_impl, retry_last_response_impl};

use crate::pipeline_helpers::{
    add_input_and_save, create_test_state_with_map, latest_state, save_state,
    wait_for_generation_complete,
};
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;

fn base_data() -> chronicler_engine::test_support::TestData {
    TestDataBuilder::default_test()
        .world(crate::fixtures::create_test_world())
        .map(crate::fixtures::create_test_map())
        .persona(crate::fixtures::create_test_player())
        .npcs(vec![])
        .build()
}

#[test]
fn test_sequential_execute_retry_execute() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    add_input_and_save(&app, "examine room");
    execute_action_impl(&app, "examine room".to_string());
    assert!(
        wait_for_generation_complete(&app, 1000),
        "Action A should complete"
    );

    retry_last_response_impl(&app);
    assert!(
        wait_for_generation_complete(&app, 1000),
        "Retry A should complete"
    );

    add_input_and_save(&app, "look around");
    execute_action_impl(&app, "look around".to_string());
    assert!(
        wait_for_generation_complete(&app, 1000),
        "Action B should complete"
    );

    let guard = latest_state(&app);
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

    let messages = app.storage().list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_sequential_execute_delete_execute() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    add_input_and_save(&app, "examine room");
    execute_action_impl(&app, "examine room".to_string());
    assert!(
        wait_for_generation_complete(&app, 1000),
        "Action A should complete"
    );

    let guard = latest_state(&app);
    let narration_id = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.id)
        .expect("Should have a narration entry");

    {
        let mut state = latest_state(&app);
        state.narrative.history.retain(|m| m.id != narration_id);
        save_state(&app, &state);
    }

    add_input_and_save(&app, "look around");
    execute_action_impl(&app, "look around".to_string());
    assert!(
        wait_for_generation_complete(&app, 1000),
        "Action B should complete"
    );

    let guard = latest_state(&app);
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    add_input_and_save(&app, "hello");
    execute_action_impl(&app, "hello".to_string());
    assert!(wait_for_generation_complete(&app, 1000));

    add_input_and_save(&app, "examine room");
    execute_action_impl(&app, "examine room".to_string());
    assert!(wait_for_generation_complete(&app, 1000));

    retry_last_response_impl(&app);
    assert!(wait_for_generation_complete(&app, 1000));

    let guard = latest_state(&app);
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    for action in ["examine room", "look around", "check inventory"] {
        add_input_and_save(&app, action);
        execute_action_impl(&app, action.to_string());
        assert!(
            wait_for_generation_complete(&app, 1000),
            "Action '{action}' should complete"
        );
    }

    let guard = latest_state(&app);
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
    let state_for_app1 = state.clone();
    let data1 = base_data();
    let app1 = SqliteTestAppBuilder::with_data(data1)
        .state_mut(move |s| *s = state_for_app1)
        .mock_backend(MockBackend::new)
        .build_service()
        .unwrap();

    add_input_and_save(&app1, "examine room");

    let data2 = base_data();
    let app2 = SqliteTestAppBuilder::with_data(data2)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    execute_action_impl(&app2, "examine room".to_string());
    assert!(wait_for_generation_complete(&app2, 1000));

    {
        let mut state = latest_state(&app2);
        state.narrative.history.clear();
        save_state(&app2, &state);
    }

    retry_last_response_impl(&app2);
    let guard = latest_state(&app2);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry with no input should not leave state generating"
    );
}

#[test]
fn test_reset_clears_history_and_state() {
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let state_for_app1 = state.clone();
    let data1 = base_data();
    let app1 = SqliteTestAppBuilder::with_data(data1)
        .state_mut(move |s| *s = state_for_app1)
        .mock_backend(MockBackend::new)
        .build_service()
        .unwrap();
    add_input_and_save(&app1, "walk to room2");

    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
            .to_string(),
    ]));

    let data2 = base_data();
    let app2 = SqliteTestAppBuilder::with_data(data2)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                quantifier.clone(),
            ))
        })
        .build_service()
        .unwrap();

    execute_action_impl(&app2, "walk to room2".to_string());
    assert!(wait_for_generation_complete(&app2, 1000));
    let guard = latest_state(&app2);
    assert_eq!(guard.movement.current_room_id, "room2");
    assert!(!guard.narrative.history().is_empty());

    let fresh_state = create_test_state_with_map();
    save_state(&app2, &fresh_state);

    let guard = latest_state(&app2);
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    add_input_and_save(&app, "examine room");
    execute_action_impl(&app, "examine room".to_string());
    assert!(wait_for_generation_complete(&app, 1000));

    let fresh_state = create_test_state_with_map();
    save_state(&app, &fresh_state);

    add_input_and_save(&app, "look around");
    execute_action_impl(&app, "look around".to_string());
    assert!(
        wait_for_generation_complete(&app, 1000),
        "Action after reset should complete"
    );

    let guard = latest_state(&app);
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(move |s| *s = state)
        .game_service_fn(move |storage| {
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    add_input_and_save(&app, "examine room");
    execute_action_impl(&app, "examine room".to_string());
    assert!(wait_for_generation_complete(&app, 1000));

    add_input_and_save(&app, "look around");
    execute_action_impl(&app, "look around".to_string());
    assert!(wait_for_generation_complete(&app, 1000));

    let guard = latest_state(&app);
    let narration_b_id = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.id)
        .expect("Should have narration B");

    {
        let mut state = latest_state(&app);
        state.narrative.history.retain(|m| m.id != narration_b_id);
        save_state(&app, &state);
    }

    add_input_and_save(&app, "check door");
    execute_action_impl(&app, "check door".to_string());
    assert!(wait_for_generation_complete(&app, 1000));

    let guard = latest_state(&app);
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
