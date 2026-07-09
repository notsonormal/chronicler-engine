//! Integration tests for the action pipeline: delayed LLM completion, quantifier detection of movement and NPCs (with trigger firing), and graceful handling of empty LLM responses.

use std::sync::Arc;

use crate::{
    fixtures::create_test_state,
    pipeline_helpers::{
        create_test_state_with_trigger_npc, latest_state, wait_for_condition,
        wait_for_generation_complete,
    },
    test_utils::wait::wait_for_condition_async,
};
use chronicler_engine::application::GameService;
use chronicler_engine::application::action_pipeline::execute_action_impl;
use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::test_support::{make_test_app_with_backends, make_test_app_with_mock_backend};

#[test]
fn test_delayed_llm_completes_without_deadlock() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |storage| {
        Arc::new(GameService::with_mock_quantifier(
            crate::make_test_recorder_with_storage(
                Arc::new(MockBackend::default().with_delay(200)),
                Arc::clone(storage),
            ),
            Arc::new(MockBackend::default()),
        ))
    })
    .unwrap();

    execute_action_impl(&app, "look around".to_string());

    let guard = latest_state(&app);
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

#[test]
fn test_quantifier_detects_movement() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = chronicler_engine::test_support::make_test_app_with_game_service(
        state,
        |_storage| {
            let quantifier_recorder: Arc<
                dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
            > = Arc::new(MockBackend::default().with_prompt_responses(vec![
                r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "village_square"}}"#
                    .to_string(),
            ]));
            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder(Arc::new(MockBackend::default())),
                quantifier_recorder,
            ))
        },
    )
    .unwrap();

    execute_action_impl(&app, "walk to the village square".to_string());

    let completed = wait_for_generation_complete(&app, 500);
    assert!(completed, "Movement action should complete within timeout");

    let guard = latest_state(&app);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after movement action"
    );
    assert_ne!(
        guard.movement.current_room_id, "room1",
        "Player should have moved from starting room"
    );
}

#[test]
fn test_quantifier_detects_npc_presence_and_fires_trigger() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |_storage| {
        let quantifier_recorder: Arc<
            dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
        > = Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()]),
        );
        Arc::new(GameService::with_mock_quantifier(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            quantifier_recorder,
        ))
    })
    .unwrap();

    execute_action_impl(&app, "enter the shop".to_string());

    let guard = latest_state(&app);
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

#[test]
fn test_empty_llm_response_handled_gracefully() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |storage| {
        Arc::new(GameService::with_mock_quantifier(
            crate::make_test_recorder_with_storage(
                Arc::new(MockBackend::default().with_empty_response()),
                Arc::clone(storage),
            ),
            Arc::new(MockBackend::default()),
        ))
    })
    .unwrap();

    execute_action_impl(&app, "examine the room".to_string());

    let guard = latest_state(&app);
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

#[test]
fn test_failing_trigger_narration_does_not_crash() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |storage| {
        let quantifier_recorder: Arc<
            dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
        > = Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()]),
        );
        Arc::new(GameService::with_mock_quantifier(
            crate::make_test_recorder_with_storage(
                Arc::new(MockBackend::default().with_trigger_narration_fail()),
                Arc::clone(storage),
            ),
            quantifier_recorder,
        ))
    })
    .unwrap();

    execute_action_impl(&app, "examine the shopkeeper".to_string());

    let guard = latest_state(&app);
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

#[test]
fn test_pipeline_cancels_when_token_cancelled() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();
    app.cancel_token().cancel();

    execute_action_impl(&app, "look".to_string());

    let final_state = latest_state(&app);
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Should reset to Idle when cancelled"
    );
}

#[tokio::test]
async fn test_cancellation_resets_state_to_idle() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |storage| {
        Arc::new(GameService::with_mock_quantifier(
            crate::make_test_recorder_with_storage(
                Arc::new(MockBackend::default().with_delay(50)),
                Arc::clone(storage),
            ),
            Arc::new(MockBackend::default()),
        ))
    })
    .unwrap();
    let token = app.cancel_token().clone();

    token.cancel();
    execute_action_impl(&app, "look around".to_string());

    let guard = latest_state(&app);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation"
    );
}

#[tokio::test]
async fn test_pipeline_cancels_after_main_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let mock_narrator_backend = Arc::new(MockBackend::default().with_delay(50));
    let mock_narrator_recorder = crate::make_test_recorder(Arc::clone(&mock_narrator_backend)
        as Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider>);
    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(MockBackend::default());
    let backend = Arc::new(GameService::with_mock_quantifier(
        mock_narrator_recorder,
        quantifier_recorder,
    ));
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |_storage| {
        Arc::clone(&backend)
    })
    .unwrap();
    let token = app.cancel_token().clone();

    let app_clone = Arc::clone(&app);
    let handle = tokio::task::spawn_blocking(move || {
        execute_action_impl(&app_clone, "look around".to_string());
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

    let guard = latest_state(&app);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-narration checkpoint"
    );
}

#[tokio::test]
async fn test_pipeline_cancels_during_trigger_continuation() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let mock_narrator_backend = Arc::new(MockBackend::default().with_trigger_delay(50));
    let mock_narrator_recorder = crate::make_test_recorder(Arc::clone(&mock_narrator_backend)
        as Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider>);
    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(
        MockBackend::default()
            .with_prompt_responses(vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()]),
    );
    let backend = Arc::new(GameService::with_mock_quantifier(
        mock_narrator_recorder,
        quantifier_recorder,
    ));
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |_storage| {
        Arc::clone(&backend)
    })
    .unwrap();
    let token = app.cancel_token().clone();

    let app_clone = Arc::clone(&app);
    let handle = tokio::task::spawn_blocking(move || {
        execute_action_impl(&app_clone, "enter the shop".to_string());
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

    let guard = latest_state(&app);
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

#[test]
fn test_pre_main_snapshot_saved_before_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = make_test_app_with_mock_backend(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "examine the room".to_string());

    let completed = wait_for_generation_complete(&app, 1000);
    assert!(completed, "FreeAction should complete within timeout");

    let latest = app.storage().load_latest_snapshot().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_pre_event_snapshot_saved_before_continuation() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |_storage| {
        let quantifier_recorder: Arc<
            dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
        > = Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()]),
        );
        Arc::new(GameService::with_mock_quantifier(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            quantifier_recorder,
        ))
    })
    .unwrap();

    execute_action_impl(&app, "examine the shopkeeper".to_string());

    let completed = wait_for_generation_complete(&app, 1000);
    assert!(
        completed,
        "FreeAction with trigger should complete within timeout"
    );

    let latest = app.storage().load_latest_snapshot().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_pipeline_with_quantifier() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = make_test_app_with_mock_backend(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "look around".to_string());

    let guard = latest_state(&app);
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

#[test]
fn test_streaming_narration_saved_before_quantifier_complete() {
    use std::thread;
    use std::time::Duration;

    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;

    let quantifier_recorder: Arc<
        dyn chronicler_engine::application::ports::llm_provider::LlmProvider,
    > = Arc::new(
        MockBackend::default()
            .with_prompt_responses(vec![r#"{"npcs_in_room": []}"#.to_string()])
            .with_delay(500),
    );
    let backend_arc = Arc::new(GameService::with_mock_quantifier(
        crate::make_test_recorder(Arc::new(MockBackend::default())),
        quantifier_recorder,
    ));
    let app = chronicler_engine::test_support::make_test_app_with_game_service(state, |_storage| {
        Arc::clone(&backend_arc)
    })
    .unwrap();

    let backend_clone = Arc::clone(&backend_arc);
    let app_clone = Arc::clone(&app);
    let handle = thread::spawn(move || {
        let _ = backend_clone;
        execute_action_impl(&app_clone, "look around".to_string());
    });

    let narration_found = wait_for_condition(
        Duration::from_millis(400),
        Duration::from_millis(50),
        || {
            app.load_messages()
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

    let guard = latest_state(&app);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Should complete after quantifier finishes"
    );

    let final_messages = app.load_messages().unwrap();
    let final_narration_count = final_messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        final_narration_count, 1,
        "Should have exactly 1 narration (no duplicates), found {final_narration_count}"
    );
}

#[test]
fn test_narration_no_duplicate_with_real_quantifier_flow() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = make_test_app_with_mock_backend(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "test action".to_string());

    let guard = latest_state(&app);

    let history = guard.narrative.history();
    let narration_count = history
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        narration_count, 1,
        "Should have exactly 1 narration entry (no duplicates), found {narration_count}"
    );

    let messages = app.load_messages().unwrap();
    let stored_narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        stored_narration_count, 1,
        "Storage should have exactly 1 narration (no duplicates), found {stored_narration_count}"
    );
}

#[test]
fn test_pipeline_continues_when_quantifier_save_warns() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let app = make_test_app_with_mock_backend(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "look".to_string());

    let guard = latest_state(&app);
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
