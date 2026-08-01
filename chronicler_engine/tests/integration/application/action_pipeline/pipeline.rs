//! Integration tests for the action pipeline: delayed LLM completion, quantifier detection of movement and NPCs (with trigger firing), and graceful handling of empty LLM responses.

use std::sync::Arc;

use crate::{
    test_utils::wait::wait_for_condition_sync, sqlite_test_app_builder::SqliteTestAppBuilder,
    test_utils::wait::wait_for_condition_async,
};
use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::TestData;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use crate::application_ext::PipelineHelpers;

fn trigger_data() -> TestData {
    TestData {
        world: Arc::new(crate::fixtures::create_test_world()),
        map: Arc::new(crate::fixtures::create_test_map()),
        persona: Arc::new(crate::fixtures::create_test_player()),
        npcs: vec![
            chronicler_engine::test_support::TestNpc::with_times_met_trigger(
                "shopkeeper",
                "Shopkeeper Sarah",
                chronicler_engine::domain::model::trigger::ComparisonOperator::Eq,
                0,
            ),
        ],
        room_npcs: vec!["shopkeeper".to_string()],
    }
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 3.3
#[test]
fn test_delayed_llm_completes_without_deadlock() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .pipeline_fn(|storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::default().with_delay(200)),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg),
                Arc::clone(settings),
            )
        })
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look around".to_string());

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after delayed action completes"
    );
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should be reset after completion"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.3
#[test]
fn test_quantifier_detects_movement() {
    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "village_square"}}"#
            .to_string(),
    ]));
    let backend = Arc::new(
        chronicler_engine::test_support::make_test_pipeline_with_mock_quantifier(
            Arc::new(chronicler_engine::adapters::driven::storage::Storage::new_in_memory()),
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            quantifier_recorder,
        ),
    );
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .pipeline_fn(move |_storage, _pg, _settings, _token| backend.as_ref().clone())
        .build_with_state()
        .unwrap();

    app.pipeline
        .execute_action("walk to the village square".to_string());

    let completed = app.wait_for_generation_complete(500);
    assert!(completed, "Movement action should complete within timeout");

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after movement action"
    );
    assert_ne!(
        guard.movement.current_room_id, "room1",
        "Player should have moved from starting room"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.4
#[test]
fn test_quantifier_detects_npc_presence_and_fires_trigger() {
    let data = trigger_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(|state| {
            state.narrative.history.clear();
            state.narrative.input_buffer.status = GenerationStatus::Generating;
            if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
                encounter.times_met = 0;
            }
        })
        .pipeline_fn(|_storage, pg, settings, token| {
            let quantifier_recorder: Arc<
                dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
            > =
                Arc::new(MockBackend::default().with_prompt_responses(vec![
                    r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string(),
                ]));
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder(Arc::new(MockBackend::default())),
                quantifier_recorder,
                Arc::clone(pg),
                Arc::clone(settings),
            )
        })
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("enter the shop".to_string());

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after trigger action"
    );

    let has_event = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.event_header.is_some());
    assert!(has_event, "Trigger should add an event header");

    let narration_count = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 2,
        "Should have main narration + trigger continuation narration"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 2.3
#[test]
fn test_empty_llm_response_handled_gracefully() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .pipeline_fn(|storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::default().with_empty_response()),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg),
                Arc::clone(settings),
            )
        })
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("examine the room".to_string());

    let guard = app.latest_state();
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")
        ),
        "Status should be Error after empty LLM response: {:?}",
        guard.narrative.input_buffer.status
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        !has_narration,
        "Empty narration should NOT be added to history"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 2.4
#[test]
fn test_failing_trigger_narration_does_not_crash() {
    let data = trigger_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(|state| {
            state.narrative.history.clear();
            state.narrative.input_buffer.status = GenerationStatus::Generating;
            if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
                encounter.times_met = 0;
            }
        })
        .pipeline_fn(|storage, pg, settings, token| {
            let quantifier_recorder: Arc<
                dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
            > =
                Arc::new(MockBackend::default().with_prompt_responses(vec![
                    r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string(),
                ]));
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::default().with_trigger_narration_fail()),
                    Arc::clone(storage),
                ),
                quantifier_recorder,
                Arc::clone(pg),
                Arc::clone(settings),
            )
        })
        .build_with_state()
        .unwrap();

    app.pipeline
        .execute_action("examine the shopkeeper".to_string());

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after trigger narration failure"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "Main narration should exist even when trigger narration failed"
    );

    let has_trigger_error = guard.narrative.history().into_iter().any(|e| {
        e.message_type == MessageType::System && e.text.contains("Trigger narration failed")
    });
    assert!(
        has_trigger_error,
        "Trigger narration failure should be logged"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 4.1
#[test]
fn test_pipeline_cancels_when_token_cancelled() {
    let app = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.shutdown_token.cancel();

    app.pipeline.execute_action("look".to_string());

    let final_state = app.latest_state();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Should reset to Idle when cancelled"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 4.1
#[tokio::test]
async fn test_cancellation_resets_state_to_idle() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .pipeline_fn(|storage, pg, settings, token| {
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder_with_storage(
                    Arc::new(MockBackend::default().with_delay(50)),
                    Arc::clone(storage),
                ),
                Arc::new(MockBackend::default()),
                Arc::clone(pg),
                Arc::clone(settings),
            )
        })
        .build_with_state()
        .unwrap();

    let token = app.shutdown_token.clone().clone();

    token.cancel();
    app.pipeline.execute_action("look around".to_string());

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 4.2
#[tokio::test]
async fn test_pipeline_cancels_after_main_narration() {
    let mock_narrator_backend = Arc::new(MockBackend::default().with_delay(50));
    let mock_narrator_recorder = crate::make_test_recorder(Arc::clone(&mock_narrator_backend)
        as Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider>);
    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(MockBackend::default());
    let backend = Arc::new(
        chronicler_engine::test_support::make_test_pipeline_with_mock_quantifier(
            Arc::new(chronicler_engine::adapters::driven::storage::Storage::new_in_memory()),
            mock_narrator_recorder,
            quantifier_recorder,
        ),
    );
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .pipeline_fn(move |_storage, _pg, _settings, _token| backend.as_ref().clone())
        .build_with_state()
        .unwrap();

    let token = app.shutdown_token.clone().clone();

    let app_clone = app.clone();
    let handle = tokio::task::spawn_blocking(move || {
        app_clone.pipeline.execute_action("look around".to_string());
    });

    assert!(
        wait_for_condition_async(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(50),
            || async {
                mock_narrator_backend
                    .narration_started
                    .load(std::sync::atomic::Ordering::SeqCst)
            }
        )
        .await,
        "narration should start within timeout"
    );
    token.cancel();

    handle.await.unwrap();

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-narration checkpoint"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 4.3
#[tokio::test]
async fn test_pipeline_cancels_during_trigger_continuation() {
    let data = trigger_data();
    let mock_narrator_backend = Arc::new(MockBackend::default().with_trigger_delay(50));
    let mock_narrator_recorder = crate::make_test_recorder(Arc::clone(&mock_narrator_backend)
        as Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider>);
    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(
        MockBackend::default()
            .with_prompt_responses(vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()]),
    );
    let backend = Arc::new(
        chronicler_engine::test_support::make_test_pipeline_with_mock_quantifier(
            Arc::new(chronicler_engine::adapters::driven::storage::Storage::new_in_memory()),
            mock_narrator_recorder,
            quantifier_recorder,
        ),
    );
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(|state| {
            state.narrative.history.clear();
            state.narrative.input_buffer.status = GenerationStatus::Generating;
            if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
                encounter.times_met = 0;
            }
        })
        .pipeline_fn(move |_storage, _pg, _settings, _token| backend.as_ref().clone())
        .build_with_state()
        .unwrap();

    let token = app.shutdown_token.clone().clone();

    let app_clone = app.clone();
    let handle = tokio::task::spawn_blocking(move || {
        app_clone
            .pipeline
            .execute_action("enter the shop".to_string());
    });

    assert!(
        wait_for_condition_async(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(50),
            || async {
                mock_narrator_backend
                    .trigger_started
                    .load(std::sync::atomic::Ordering::SeqCst)
            }
        )
        .await,
        "trigger should start within timeout"
    );
    token.cancel();

    handle.await.unwrap();

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-trigger checkpoint"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "Main narration should be preserved");
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 5.1
#[test]
fn test_pre_main_snapshot_saved_before_narration() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Idle, GenerationPhase::default())
        .mock_backend(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("examine the room".to_string());

    let completed = app.wait_for_generation_complete(1000);
    assert!(completed, "FreeAction should complete within timeout");

    let latest = app.storage.load_latest_snapshot().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 5.2
#[test]
fn test_pre_event_snapshot_saved_before_continuation() {
    let data = trigger_data();
    let app = SqliteTestAppBuilder::with_data(data)
        .state_mut(|state| {
            state.narrative.history.clear();
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
                encounter.times_met = 0;
            }
        })
        .pipeline_fn(|_storage, pg, settings, token| {
            let quantifier_recorder: Arc<
                dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
            > =
                Arc::new(MockBackend::default().with_prompt_responses(vec![
                    r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string(),
                ]));
            chronicler_engine::application::pipeline::pipeline::ActionPipeline::with_mock_quantifier(
                token,
    crate::make_test_recorder(Arc::new(MockBackend::default())),
                quantifier_recorder,
                Arc::clone(pg),
                Arc::clone(settings),
            )
        })
        .build_with_state()
        .unwrap();

    app.pipeline
        .execute_action("examine the shopkeeper".to_string());

    let completed = app.wait_for_generation_complete(1000);
    assert!(
        completed,
        "FreeAction with trigger should complete within timeout"
    );

    let latest = app.storage.load_latest_snapshot().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.1
#[test]
fn test_pipeline_with_quantifier() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .mock_backend(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look around".to_string());

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Should complete with quantifier backend"
    );
    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "Should produce narration with quantifier");
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 3.2
#[test]
fn test_streaming_narration_saved_before_quantifier_complete() {
    use std::thread;
    use std::time::Duration;

    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(
        MockBackend::default()
            .with_prompt_responses(vec![r#"{"npcs_in_room": []}"#.to_string()])
            .with_delay(500),
    );
    let backend_for_builder = Arc::new(
        chronicler_engine::test_support::make_test_pipeline_with_mock_quantifier(
            Arc::new(chronicler_engine::adapters::driven::storage::Storage::new_in_memory()),
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            quantifier_recorder,
        ),
    );
    let backend_arc = Arc::clone(&backend_for_builder);
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .pipeline_fn(move |_storage, _pg, _settings, _token| backend_for_builder.as_ref().clone())
        .build_with_state()
        .unwrap();

    let pg = &app.persistence_gate;

    let backend_clone = Arc::clone(&backend_arc);
    let app_clone = app.clone();
    let handle = thread::spawn(move || {
        let _ = backend_clone;
        app_clone.pipeline.execute_action("look around".to_string());
    });

    let narration_found = wait_for_condition_sync(
        Duration::from_millis(400),
        Duration::from_millis(50),
        || {
            pg.load_messages()
                .map(|msgs| {
                    msgs.iter()
                        .any(|m| m.message_type == MessageType::Narration)
                })
                .unwrap_or(false)
        },
    );

    assert!(
        narration_found,
        "Narration should be saved before quantifier completes (quantifier takes 500ms)"
    );

    handle.join().expect("Action thread should complete");

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Should complete after quantifier finishes"
    );

    let final_messages = pg.load_messages().unwrap();
    let final_narration_count = final_messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        final_narration_count, 1,
        "Should have exactly 1 narration (no duplicates), found {final_narration_count}"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 3.2
#[test]
fn test_narration_no_duplicate_with_real_quantifier_flow() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .mock_backend(MockBackend::default)
        .build_with_state()
        .unwrap();

    let pg = &app.persistence_gate;

    app.pipeline.execute_action("test action".to_string());

    let guard = app.latest_state();

    let history = guard.narrative.history();
    let narration_count = history
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        narration_count, 1,
        "Should have exactly 1 narration entry (no duplicates), found {narration_count}"
    );

    let messages = pg.load_messages().unwrap();
    let stored_narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        stored_narration_count, 1,
        "Storage should have exactly 1 narration (no duplicates), found {stored_narration_count}"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.1
#[test]
fn test_pipeline_continues_when_quantifier_save_warns() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Generating, GenerationPhase::default())
        .mock_backend(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Pipeline should complete even if quantifier save has warnings"
    );

    let has_narration = guard
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "Narration should be saved regardless of quantifier"
    );
}
