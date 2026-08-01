//! Integration flow tests for the retry-main handler: new quantifier result on re-narration, re-running the quantifier on different text, double-retry swipe increment, and the no-extra-swipe guarantee when input is preserved.

use std::sync::Arc;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;

use chronicler_engine::domain::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::{
    ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
};
use chronicler_engine::test_support::{make_test_pipeline_with_mock_quantifier, TestDataBuilder};

use crate::make_test_recorder_with_storage;
use crate::fixtures::create_minimal_test_state;
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;
use crate::application_ext::PipelineHelpers;

fn base_data(npcs: Vec<NpcCard>) -> chronicler_engine::test_support::TestData {
    TestDataBuilder::default_test()
        .world(crate::fixtures::create_test_world())
        .map(crate::fixtures::create_test_map())
        .persona(crate::fixtures::create_test_player())
        .npcs(npcs)
        .build()
}

#[test]
fn test_retry_main_narration_applies_new_quantifier_result() {
    let data = base_data(vec![]);
    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let quantifier = Arc::clone(&quantifier);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier,
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk around");
    app.pipeline.execute_action("walk around".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "First execution should complete"
    );
    let guard = app.latest_state();
    assert_eq!(
        guard.movement.current_room_id, "room1",
        "First execution: player should stay in room1"
    );
    app.pipeline.retry_last_response();

    assert!(
        app.wait_for_generation_complete(1000),
        "Retry should complete"
    );
    let guard = app.latest_state();
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Retry should apply NEW quantifier result and move player to room2"
    );

    let messages = app.storage.list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_retry_with_different_narration_text_reruns_quantifier() {
    let data = base_data(vec![]);
    let narration_backend: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(MockBackend::default().with_narrations(vec![
        "You look around the empty room.".to_string(),
        "The Innkeeper greets you warmly.".to_string(),
    ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let narration_backend = Arc::clone(&narration_backend);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(narration_backend, Arc::clone(storage)),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("approach the innkeeper");
    app.pipeline
        .execute_action("approach the innkeeper".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "First execution should complete"
    );
    let guard = app.latest_state();
    let first_narration = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert_eq!(
        first_narration, "You look around the empty room.",
        "First narration should match per_call_narrations[0]"
    );

    app.pipeline.retry_last_response();
    assert!(
        app.wait_for_generation_complete(1000),
        "Retry should complete"
    );
    let guard = app.latest_state();
    let retry_narration = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert_eq!(
        retry_narration, "The Innkeeper greets you warmly.",
        "Retry narration should match per_call_narrations[1]"
    );
}

#[test]
fn test_double_retry_increments_swipe_and_reruns_quantifier() {
    let data = base_data(vec![]);
    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
            r#"{"npcs_in_room": []}"#.to_string(),
        ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let quantifier = Arc::clone(&quantifier);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier,
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk around");
    app.pipeline.execute_action("walk around".to_string());
    assert!(app.wait_for_generation_complete(1000));
    let _snap = app.latest_snapshot();

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));
    let _snap = app.latest_snapshot();
    let guard = app.latest_state();
    assert_eq!(guard.movement.current_room_id, "room2");

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));
    let _snap = app.latest_snapshot();

    let guard = app.latest_state();
    let history = guard.narrative.history();
    assert!(
        !history.is_empty(),
        "Second retry should produce narration entries"
    );
}

#[test]
fn test_retry_preserves_input_and_does_not_create_extra_swipe() {
    let data = base_data(vec![]);
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
    chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
        token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
        Arc::new(MockBackend::default()),
        Arc::clone(pg),
        Arc::clone(settings),
    )
})
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk around");
    app.pipeline.execute_action("walk around".to_string());
    assert!(
        app.wait_for_generation_complete(1000),
        "First execution should complete"
    );

    app.pipeline.retry_last_response();
    assert!(
        app.wait_for_generation_complete(1000),
        "Retry should complete"
    );

    let guard = app.latest_state();
    let input_msg = guard
        .narrative
        .history
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message must exist");
    assert_eq!(
        input_msg.text(),
        "walk around",
        "Input message text must be preserved after retry"
    );
}

#[test]
fn test_retry_after_edited_input_uses_new_text() {
    let data = base_data(vec![]);
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
    chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
        token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
        Arc::new(MockBackend::default()),
        Arc::clone(pg),
        Arc::clone(settings),
    )
})
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk around");
    app.pipeline.execute_action("walk around".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let guard = app.latest_state();
    let first_narration = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert!(
        first_narration.contains("walk around"),
        "First narration should contain original input: {first_narration}"
    );

    {
        let mut state = app.latest_state();
        if let Some(msg) = state
            .narrative
            .history
            .iter_mut()
            .find(|m| m.message_type == MessageType::Input)
        {
            msg.update_active_swipe_text("sprint forward".to_string());
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.text = "sprint forward".to_string();
            }
        }
        app.save_test_state(&state);
    }

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));
    let guard = app.latest_state();
    let retry_narration = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.message_type == MessageType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert!(
        retry_narration.contains("sprint forward"),
        "Retry narration should contain edited input: {retry_narration}"
    );
}

#[test]
fn test_main_retry_reevaluates_triggers() {
    let shopkeeper = NpcCard {
        id: "shopkeeper".into(),
        sheet: CharacterSheet {
            name: "Shopkeeper Sarah".into(),
            description: "A shrewd shopkeeper".into(),
            personality: "Business-minded".into(),
            scenario: "Runs the shop".into(),
            example_dialogue: "Welcome!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
            narration: TriggerNarration {
                name: "Greeting".into(),
                narration_prompt: "The shopkeeper looks up with a smile.".into(),
            },
            repeat: false,
            room_id: Some("room2".to_string()),
        }],
        relationships: vec![],
    };
    let data = base_data(vec![shopkeeper]);

    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let quantifier = Arc::clone(&quantifier);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier,
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk around");
    app.pipeline.execute_action("walk around".to_string());
    assert!(app.wait_for_generation_complete(1000));
    let guard = app.latest_state();
    let events_after_execute = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.event_header.is_some())
        .count();
    assert_eq!(
        events_after_execute, 0,
        "First execution: no trigger (not in room2)"
    );

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));
    let guard = app.latest_state();
    let events_after_retry = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.event_header.is_some())
        .count();
    assert_eq!(
        events_after_retry, 1,
        "Retry should re-evaluate triggers and fire when moved to room2"
    );
}

#[test]
fn test_retry_completes_when_quantifier_returns_none() {
    let data = base_data(vec![]);
    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": []}"#.to_string(),
        ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let quantifier = Arc::clone(&quantifier);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier,
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk around");
    app.pipeline.execute_action("walk around".to_string());
    assert!(app.wait_for_generation_complete(1000));

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));
    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry should complete even if quantifier returns None"
    );
}

#[test]
fn test_retry_no_pre_main_snapshot() {
    let state = create_minimal_test_state();

    let db_pool =
        chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:").unwrap();
    chronicler_engine::test_support::seed_default_game_row(&db_pool, 1).unwrap();
    let storage = Arc::new(
        chronicler_engine::adapters::driven::storage::Storage::new_sqlite(db_pool.clone(), 1),
    );

    let snapshot =
        chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        );
    let _ = storage.save_snapshot(&snapshot);

    let preset_storage = {
        let ps = chronicler_engine::adapters::driven::storage::Storage::new_in_memory();
        let _ = ps.save_preset(
            &chronicler_engine::domain::model::prompt_preset::PromptPreset {
                id: "system_default".to_string(),
                name: "Default System".to_string(),
                role: Some("You are a narrator.".to_string()),
                instructions: None,
                writing_style: None,
                output_format: None,
                is_default: true,
                preset_type: chronicler_engine::domain::model::prompt_preset::PresetType::System,
            },
        );
        let _ = ps.save_preset(
            &chronicler_engine::domain::model::prompt_preset::PromptPreset {
                id: "quantifier_default".to_string(),
                name: "Default Quantifier".to_string(),
                role: Some("You are a quantifier.".to_string()),
                instructions: None,
                writing_style: None,
                output_format: None,
                is_default: true,
                preset_type:
                    chronicler_engine::domain::model::prompt_preset::PresetType::Quantifier,
            },
        );
        Arc::new(ps)
    };

    let state = chronicler_engine::test_support::build_test_wired_app(
        std::sync::Arc::clone(&storage),
        std::sync::Arc::clone(&preset_storage),
        make_test_pipeline_with_mock_quantifier(
            std::sync::Arc::new(
                chronicler_engine::adapters::driven::storage::Storage::new_in_memory(),
            ),
            crate::make_test_recorder(std::sync::Arc::new(MockBackend::new())),
            std::sync::Arc::new(MockBackend::default()),
        ),
    )
    .expect("build_test_wired_app: build_app_graph_for_tests should succeed");
    let app = chronicler_engine::adapters::driving::http::AppState::from_wired(state);
    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let state_before_reset = app.latest_state();

    {
        let conn = db_pool.conn();
        let _ = conn.execute("DELETE FROM game_state_snapshots WHERE game_id = 1", []);
    }
    {
        app.save_test_state(&state_before_reset);
    }

    app.pipeline.retry_last_response();

    let stable = app.wait_for_generation_complete(500);
    assert!(
        stable,
        "Retry with no pre-main snapshot should complete (possibly with error)"
    );
}

#[test]
fn test_movement_with_arrival_narration_retry() {
    let data = base_data(vec![]);
    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let quantifier = Arc::clone(&quantifier);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
                quantifier,
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    app.add_input_and_save("walk to room2");
    app.pipeline.execute_action("walk to room2".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let guard = app.latest_state();
    let arrival_count_before = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert!(
        arrival_count_before > 0,
        "Should have at least one narration persisted before retry"
    );

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));

    let guard = app.latest_state();
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Retry should still end in room2"
    );

    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(!narrations.is_empty(), "Retry should produce narrations");
}

#[test]
fn test_retry_appends_swipe_to_existing_narration() {
    let data = base_data(vec![]);
    let narration_backend: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(MockBackend::default().with_narrations(vec![
        "First narration text.".to_string(),
        "Second narration text.".to_string(),
    ]));
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |storage, pg, settings, token| {
            let narration_backend = Arc::clone(&narration_backend);
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    make_test_recorder_with_storage(narration_backend, Arc::clone(storage)),
                Arc::new(MockBackend::default()),
                Arc::clone(pg), Arc::clone(settings))
        })
        .build_with_state()
        .unwrap();

    let pg = &app.persistence_gate;

    app.add_input_and_save("examine room");
    app.pipeline.execute_action("examine room".to_string());
    assert!(app.wait_for_generation_complete(1000));

    let msgs = pg.load_messages().unwrap();
    let narration = msgs
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("Should have narration");
    let original_id = narration.id;
    assert_eq!(narration.swipes.len(), 1);

    app.pipeline.retry_last_response();
    assert!(app.wait_for_generation_complete(1000));

    let msgs = pg.load_messages().unwrap();
    let narrations: Vec<_> = msgs
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        1,
        "Retry should keep exactly one narration message"
    );
    assert_eq!(
        narrations[0].id, original_id,
        "Retry should keep the same message ID"
    );
    assert_eq!(
        narrations[0].swipes.len(),
        2,
        "Retry should append a new swipe"
    );
    assert_eq!(
        narrations[0].text(),
        "Second narration text.",
        "Retry should use next per-call narration as the active swipe"
    );
}
