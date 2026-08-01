//! Integration flow tests for action sequencing: execute→retry→execute, execute→delete→execute, async action ordering, and three-action sequence under realistic state churn.

use std::sync::Arc;

use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::TestDataBuilder;

use crate::fixtures::create_minimal_test_state;
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;
use crate::application_ext::PipelineHelpers;

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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "Action A should complete"
    );

    app.pipeline.retry_last_response();
    assert!(
        app.wait_for_generation_complete(1000),
        "Retry A should complete"
    );

    app.add_input_and_save("look around");
    app.pipeline.execute_action("look around".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "Action B should complete"
    );

    let guard = app.latest_state();
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

    let messages = app.storage.list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_sequential_execute_delete_execute() {
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "Action A should complete"
    );

    let guard = app.latest_state();
    let narration_id = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.id)
        .expect("Should have a narration entry");

    {
        let mut state = app.latest_state();
        state.narrative.history.retain(|m| m.id != narration_id);
        app.save_test_state(&state);
    }

    app.add_input_and_save("look around");
    app.pipeline.execute_action("look around".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "Action B should complete"
    );

    let guard = app.latest_state();
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("hello");
    app.pipeline.execute_action("hello".to_string());
    assert!(app.wait_for_generation_complete(1000));

    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(app.wait_for_generation_complete(1000));

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));

    let guard = app.latest_state();
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    for action in ["examine room", "look around", "check inventory"] {
        app.add_input_and_save(action);
        app.pipeline.execute_action(action.to_string());
        assert!(
            app.wait_for_generation_complete(1000),
            "Action '{action}' should complete"
        );
    }

    let guard = app.latest_state();
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
    let data1 = base_data();
    let app1 = SqliteTestAppBuilder::with_data(data1)
        .mock_backend(MockBackend::new)
        .build_with_state()
        .unwrap();

    app1.add_input_and_save("examine room");

    let data2 = base_data();
    let app2 = SqliteTestAppBuilder::with_data(data2)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app2.pipeline.execute_action("examine room".to_string());
    assert!(app2.wait_for_generation_complete(1000));

    {
        let mut state = app2.latest_state();
        state.narrative.history.clear();
        app2.save_test_state(&state);
    }

    app2.pipeline.retry_last_response();
    let guard = app2.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry with no input should not leave state generating"
    );
}

#[test]
fn test_reset_clears_history_and_state() {
    let data1 = base_data();
    let app1 = SqliteTestAppBuilder::with_data(data1)
        .mock_backend(MockBackend::new)
        .build_with_state()
        .unwrap();

    app1.add_input_and_save("walk to room2");

    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
            .to_string(),
    ]));

    let data2 = base_data();
    let app2 = SqliteTestAppBuilder::with_data(data2)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                quantifier.clone(),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app2.pipeline.execute_action("walk to room2".to_string());
    assert!(app2.wait_for_generation_complete(1000));
    let guard = app2.latest_state();
    assert_eq!(guard.movement.current_room_id, "room2");
    assert!(!guard.narrative.history().is_empty());

    let fresh_state = create_minimal_test_state();
    app2.save_test_state(&fresh_state);

    let guard = app2.latest_state();
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let fresh_state = create_minimal_test_state();
    app.save_test_state(&fresh_state);

    app.add_input_and_save("look around");
    app.pipeline.execute_action("look around".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "Action after reset should complete"
    );

    let guard = app.latest_state();
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
    let data = base_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::new()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(app.wait_for_generation_complete(1000));

    app.add_input_and_save("look around");
    app.pipeline.execute_action("look around".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let guard = app.latest_state();
    let narration_b_id = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.id)
        .expect("Should have narration B");

    {
        let mut state = app.latest_state();
        state.narrative.history.retain(|m| m.id != narration_b_id);
        app.save_test_state(&state);
    }

    app.add_input_and_save("check door");
    app.pipeline.execute_action("check door".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let guard = app.latest_state();
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
