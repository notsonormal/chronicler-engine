//! [DOC: docs/reference/testing.md]

use std::sync::Arc;

use chronicler_engine::application::action_pipeline::{
    execute_action_impl, retry_last_response_impl,
};
use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::state::{
    GenerationPhase, GenerationStatus, LogType, StoredTriggerContext,
};
use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context;
use pipeline_helpers::{
    create_test_state_with_trigger_npc, latest_state, wait_for_generation_complete,
};

#[path = "helpers/pipeline_helpers.rs"]
mod pipeline_helpers;
mod test_data;

use test_data::create_test_state;

fn working_backend() -> DefaultGameService {
    DefaultGameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default())
}

fn failing_backend() -> DefaultGameService {
    DefaultGameService::with_backends(Arc::new(MockBackend::failing()), AgentRegistry::default())
}

// ─── execute_action_impl ───────────────────────────────────────────────────

#[test]
fn test_pipeline_executes_and_persists_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(
        has_narration,
        "Pipeline should generate and persist narration"
    );
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Status should not be generating after completion"
    );
}

#[test]
fn test_pipeline_persists_input_before_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    // Pre-seed input log (server handler responsibility)
    state.add_log(
        "examine the room".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    let entries: Vec<_> = final_state.narrative.history().into_iter().collect();
    let input_idx = entries
        .iter()
        .position(|e| e.log_type == LogType::Input && e.text == "examine the room");
    let narration_idx = entries
        .iter()
        .position(|e| e.log_type == LogType::Narration);
    assert!(input_idx.is_some(), "Input should be persisted");
    assert!(narration_idx.is_some(), "Narration should be persisted");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before narration"
    );
}

#[test]
fn test_pipeline_handles_room_not_found() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.movement.current_room_id = "non_existent_room".to_string();
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should reset generating status when room not found"
    );
}

#[test]
fn test_pipeline_handles_llm_failure() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    let backend = failing_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    assert!(
        final_state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "Should set error status when LLM fails"
    );
}

#[test]
fn test_pipeline_cancels_when_token_cancelled() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    ctx.cancel_token.cancel();
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Should reset to Idle when cancelled"
    );
}

#[test]
fn test_pipeline_clears_last_trigger() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.last_trigger = Some(StoredTriggerContext {
        trigger_name: "Old".to_string(),
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_repeat: false,
        trigger_narration_prompt: "The old trigger fires.".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared"
    );
}

// ─── retry_last_response_impl ──────────────────────────────────────────────

#[test]
fn test_retry_finds_last_input_and_runs_pipeline() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );
    let after_first = latest_state(&ctx);
    let first_narration_count = after_first
        .narrative
        .history()
        .iter()
        .filter(|e| e.log_type == LogType::Narration)
        .count();
    assert_eq!(first_narration_count, 1);

    retry_last_response_impl(&backend, ctx.clone());

    let after_retry = latest_state(&ctx);
    let retry_narration_count = after_retry
        .narrative
        .history()
        .iter()
        .filter(|e| e.log_type == LogType::Narration)
        .count();
    assert_eq!(
        retry_narration_count, 1,
        "Retry should replace old narration, not append another"
    );
}

#[test]
fn test_retry_with_empty_history_is_noop() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    let backend = working_backend();

    retry_last_response_impl(&backend, ctx.clone());

    let final_state = latest_state(&ctx);
    assert!(final_state.narrative.history().is_empty());
}

#[test]
fn test_retry_after_llm_failure_succeeds() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    let ctx = make_test_context(state);
    let failing = failing_backend();

    execute_action_impl(
        &failing,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );
    let after_fail = latest_state(&ctx);
    assert!(
        after_fail
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
    );

    let working = working_backend();
    retry_last_response_impl(&working, ctx.clone());

    let after_retry = latest_state(&ctx);
    assert!(
        !after_retry.narrative.input_buffer.status.is_generating(),
        "Retry should complete: {:?}",
        after_retry.narrative.input_buffer.status
    );
}

// Moved from game_service/advanced.rs

#[test]
fn test_delayed_llm_completes_without_deadlock() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(200)),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
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
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "village_square"}}"#.to_string()],
            ..Default::default()
        }),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "walk to the village square".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 500);
    assert!(completed, "Movement action should complete within timeout");

    let guard = latest_state(&ctx);
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
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "enter the shop".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
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
        .filter(|e| e.log_type == LogType::Narration)
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
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
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
        .any(|e| e.log_type == LogType::Narration);
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
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_failing_trigger_narration()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the shopkeeper".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after trigger narration failure"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(
        has_narration,
        "Main narration should exist even when trigger narration failed"
    );

    let has_trigger_error = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::System && e.text.contains("Trigger narration failed"));
    assert!(
        has_trigger_error,
        "Trigger narration failure should be logged"
    );
}

#[tokio::test]
async fn test_cancellation_resets_state_to_idle() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(50)),
        Arc::new(MockBackend::default()),
    );
    let token = ctx.cancel_token.clone();

    // Cancel before execution
    token.cancel();
    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
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
    let ctx = make_test_context(state);
    let backend = Arc::new(DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(50)),
        Arc::new(MockBackend::default()),
    ));
    let token = ctx.cancel_token.clone();

    let ctx_clone = ctx.clone();
    let backend_clone = Arc::clone(&backend);
    let handle = tokio::task::spawn_blocking(move || {
        execute_action_impl(
            &*backend_clone,
            ctx_clone.clone(),
            "look around".to_string(),
            "Player".to_string(),
        );
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    token.cancel();

    handle.await.unwrap();

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-narration checkpoint"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(
        !has_narration,
        "Narration should be discarded when cancelled after main LLM call"
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
    let ctx = make_test_context(state);
    let backend = Arc::new(DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_trigger_delay(50)),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    ));
    let token = ctx.cancel_token.clone();

    let ctx_clone = ctx.clone();
    let backend_clone = Arc::clone(&backend);
    let handle = tokio::task::spawn_blocking(move || {
        execute_action_impl(
            &*backend_clone,
            ctx_clone.clone(),
            "enter the shop".to_string(),
            "Player".to_string(),
        );
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    token.cancel();

    handle.await.unwrap();

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-trigger checkpoint"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(has_narration, "Main narration should be preserved");

    let has_event = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.event_header.is_some());
    assert!(
        !has_event,
        "Trigger event should be discarded when cancelled after trigger LLM call"
    );
}

#[test]
fn test_retry_no_snapshot() {
    let ctx = make_test_context(create_test_state());
    ctx.snapshot_storage.reset().unwrap();

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());
}

#[test]
fn test_retry_no_input_text() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log("System boot".to_string(), None, LogType::System);
    state.add_log("You see a room.".to_string(), None, LogType::Narration);

    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
    assert_eq!(guard.narrative.history().len(), 2);
}

#[test]
fn test_retry_room_not_found() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.movement.current_room_id = "non_existent_room".to_string();

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("Room not found")
        ),
        "Expected room not found error: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_llm_error() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Expected error status after failing LLM: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_empty_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")
        ),
        "Expected empty response error: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_pre_main_snapshot_saved_before_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "FreeAction should complete within timeout");

    let latest = ctx.snapshot_storage.load_latest().unwrap().unwrap();
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
    let ctx = make_test_context(state);

    let quantifier = MockBackend {
        per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
        ..Default::default()
    };

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(quantifier),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the shopkeeper".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(
        completed,
        "FreeAction with trigger should complete within timeout"
    );

    let latest = ctx.snapshot_storage.load_latest().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_retry_main_narration_uses_pre_main_snapshot() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.narrative.input_buffer.status = GenerationStatus::Idle;

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Retry should complete within timeout");

    let guard = latest_state(&ctx);
    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Narration)
        .collect();
    assert!(!narrations.is_empty(), "Retry should generate narration");
}

#[test]
fn test_retry_event_continuation_uses_pre_event_snapshot() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.add_log(
        "You look around the shop.".to_string(),
        None,
        LogType::Narration,
    );
    state.narrative.pending_event = Some("Greeting".to_string());
    state.add_log(
        "The shopkeeper looks up with a smile.".to_string(),
        None,
        LogType::Narration,
    );
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    state.narrative.last_trigger = Some(StoredTriggerContext {
        npc_id: "shopkeeper".to_string(),
        trigger_idx: 0,
        trigger_name: "Greeting".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "The shopkeeper looks up with a smile.".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });

    let ctx = make_test_context(state.clone());
    let pre_event = GameStateSnapshot::from_game_state(&state);
    let pre_event_id = ctx.snapshot_storage.save(&pre_event).unwrap();

    if let Some(last) = state.narrative.history.last_mut() {
        last.event_header = Some("Greeting".to_string());
    }

    let final_snap = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&final_snap);

    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.log_type == LogType::Narration && msg.event_header.is_none() {
            msg.snapshot_id = Some(pre_event_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Event retry should complete within timeout");

    let guard = latest_state(&ctx);
    let main_narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Narration)
        .collect();
    assert!(
        !main_narrations.is_empty(),
        "Should have at least one narration after retry"
    );
}

// New tests for gaps

#[test]
fn test_pipeline_phase_transitions() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after successful completion"
    );
}

#[test]
fn test_pipeline_phase_stays_narrating_on_error() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let backend = failing_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::Narrating,
        "Phase should remain Narrating after failed narration"
    );
}

#[test]
fn test_pipeline_empty_input() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    let backend = failing_backend();

    execute_action_impl(&backend, ctx.clone(), "".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        guard
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
            || !guard.narrative.input_buffer.status.is_generating(),
        "Empty input should complete without panic: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_pipeline_with_quantifier() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Should complete with quantifier backend"
    );
    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(has_narration, "Should produce narration with quantifier");
}
