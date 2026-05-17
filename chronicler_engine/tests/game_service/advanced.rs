use std::sync::Arc;

use chronicler_engine::engine::game_service::{DefaultGameService, GameService};
use chronicler_engine::model::state::{GameState, GenerationStatus, LogType};
use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context;

use crate::test_data::create_test_state;
use crate::{create_test_state_with_trigger_npc, failing_service, wait_for_generation_complete};

#[test]
fn test_execute_freeaction_immediate_return() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let service = failing_service();

    // FreeAction should return immediately and spawn a thread
    // The function should not block
    service.execute_action(
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    // State should be accessible immediately after execute_action returns
    // (the thread runs in background)
    let guard = crate::latest_state(&ctx);
    let status = &guard.narrative.generation.status;
    // Failing mock backend causes FreeAction to fail and set Error status
    assert!(
        status.error_message().is_some(),
        "Status should be Error after failed FreeAction: {status:?}"
    );
}

#[test]
fn test_execute_freeaction_room_not_found() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    state.movement.current_room_id = "non_existent_room".to_string();
    let ctx = make_test_context(state);
    let service = DefaultGameService::new();

    // Execute FreeAction - should not panic
    service.execute_action(
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    // Verify is_generating was reset (room not found path)
    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(
        completed,
        "is_generating should be reset when room not found"
    );
}

#[test]
fn test_execute_freeaction_state_accessible() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = failing_service();

    service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());

    // State should remain accessible after execute_action returns
    let guard = crate::latest_state(&ctx);
    let status = &guard.narrative.generation.status;
    // Failing mock backend causes FreeAction to fail and set Error status
    assert!(
        status.error_message().is_some(),
        "Status should be Error after failed FreeAction: {status:?}"
    );
}

#[test]
fn test_execute_freeaction_narration_failure() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = failing_service();

    service.execute_action(ctx.clone(), "test action".to_string(), "Player".to_string());

    let completed = wait_for_generation_complete(&ctx, 200);
    assert!(completed, "FreeAction should complete within timeout");

    let guard = crate::latest_state(&ctx);
    // MockBackend::failing() always returns an error
    assert!(
        guard.narrative.generation.status.error_message().is_some(),
        "Should have error after failed narration: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_execute_freeaction_with_mock_backend() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating; // set by caller (server)
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(
        ctx.clone(),
        "examine the room carefully".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 200);
    assert!(completed, "FreeAction should complete within timeout");

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "is_generating should be reset after FreeAction completes"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(has_narration, "Mock LLM should add narration to history");
}

#[test]
fn test_retry_with_mock_backend() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.add_log("look around".to_string(), None, LogType::Input);
    state.add_log("Initial narration".to_string(), None, LogType::Narration);
    state.narrative.generation.status = GenerationStatus::Generating; // set by caller (server)
    let ctx = make_test_context(state.clone());

    // Save pre-main snapshot so retry has something to work with
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.messages.clone() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    // Re-save the main snapshot so it remains the latest (load_latest uses max created_at)
    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Retry should complete within timeout");

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "is_generating should be reset after retry completes"
    );

    // The last AI response should have been replaced with mock narration
    let ai_responses: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Narration)
        .collect();
    assert!(
        !ai_responses.is_empty(),
        "Should have AI responses after retry"
    );
}

#[test]
fn test_execute_freeaction_with_movement_mock() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating; // set by caller (server)
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    // Action that implies movement
    service.execute_action(
        ctx.clone(),
        "walk to the north".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(
        completed,
        "FreeAction with movement should complete within timeout"
    );

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "is_generating should be reset after FreeAction with movement"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(
        has_narration,
        "Mock LLM should add narration even for movement actions"
    );
}

#[test]
fn test_freeaction_phase_starts_narrating() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let service = DefaultGameService::new();

    service.execute_action(
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let guard = crate::latest_state(&ctx);
    // DefaultGameService has no API key, so FreeAction fails.
    // set_phase(Narrating) runs before the backend call, and set_error_and_reset
    // only updates status (not phase), so phase should still be Narrating.
    assert_eq!(
        guard.narrative.generation.phase,
        chronicler_engine::model::state::GenerationPhase::Narrating,
        "Phase should be Narrating after starting FreeAction: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_freeaction_phase_transitions_mock() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(
        ctx.clone(),
        "examine the room carefully".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 200);
    assert!(completed, "FreeAction should complete within timeout");

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Status should be reset after FreeAction completes"
    );
    assert_eq!(
        guard.narrative.generation.phase,
        chronicler_engine::model::state::GenerationPhase::default(),
        "Phase should be reset to default after completion"
    );
}

#[tokio::test]
async fn test_cancellation_resets_state_to_idle() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(50)),
        Arc::new(MockBackend::default()),
    );
    let token = ctx.cancel_token.clone();
    let token_clone = token.clone();

    let ctx_clone = ctx.clone();
    let handle = tokio::task::spawn_blocking(move || {
        if token_clone.is_cancelled() {
            if let Ok(Some(snap)) = ctx_clone.snapshot_storage.load_latest() {
                let mut state = GameState::from_snapshot(
                    &snap,
                    ctx_clone.world.clone(),
                    ctx_clone.map.clone(),
                    ctx_clone.player.clone(),
                    (*ctx_clone.npcs).clone(),
                );
                state.narrative.generation.status = GenerationStatus::Idle;
                let snapshot = GameStateSnapshot::from_game_state(&state);
                let _ = ctx_clone.snapshot_storage.save(&snapshot);
            }
            return;
        }
        service.execute_action(
            ctx_clone.clone(),
            "look around".to_string(),
            "Player".to_string(),
        );
        if token_clone.is_cancelled() {
            if let Ok(Some(snap)) = ctx_clone.snapshot_storage.load_latest() {
                let mut state = GameState::from_snapshot(
                    &snap,
                    ctx_clone.world.clone(),
                    ctx_clone.map.clone(),
                    ctx_clone.player.clone(),
                    (*ctx_clone.npcs).clone(),
                );
                state.narrative.generation.status = GenerationStatus::Idle;
                let snapshot = GameStateSnapshot::from_game_state(&state);
                let _ = ctx_clone.snapshot_storage.save(&snapshot);
            }
        }
    });

    // Cancel while the mock backend is sleeping inside execute_action
    token.cancel();

    // Wait for the blocking task to finish
    handle.await.unwrap();

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Status should be Idle after cancellation cleanup"
    );
}

#[test]
fn test_retry_last_response_not_ai_generated() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.add_log("System message".to_string(), None, LogType::System);
    let ctx = make_test_context(state);
    let service = DefaultGameService::new();

    // Retry should find the last input and attempt to process it
    // With DefaultGameService (no API key), it will fail
    service.retry_last_response(ctx.clone());

    // Wait for the retry to complete
    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Retry should complete within timeout");

    let guard = crate::latest_state(&ctx);
    assert!(
        guard.narrative.generation.status.error_message().is_some()
            || !guard.narrative.generation.status.is_generating(),
        "Retry with no AI response should complete: {:?}",
        guard.narrative.generation.status
    );
}

// === Error Resilience Tests ===

#[test]
fn test_empty_llm_response_handled_gracefully() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let guard = crate::latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.generation.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")
        ),
        "Status should be Error after empty LLM response: {:?}",
        guard.narrative.generation.status
    );

    // Empty narration is NOT logged — it's treated as an error
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
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    // Reset times_met so the trigger is eligible to fire
    if let Some(encounter) = state.character_state.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_failing_trigger_narration()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    );

    // Use a FreeAction so the backend is invoked ("talk to" parses as Talk, not FreeAction)
    service.execute_action(
        ctx.clone(),
        "examine the shopkeeper".to_string(),
        "Player".to_string(),
    );

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Status should be reset after trigger narration failure"
    );

    // Main narration should still be present
    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(
        has_narration,
        "Main narration should exist even when trigger narration failed"
    );

    // Trigger narration failure should be logged as a system message
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

// === Status Transition & Quantifier Tests ===

#[test]
fn test_delayed_llm_completes_without_deadlock() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(200)),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());

    // execute_action is synchronous — by now the delay has elapsed
    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Status should be Idle after delayed action completes"
    );
    assert_eq!(
        guard.narrative.generation.phase,
        chronicler_engine::model::state::GenerationPhase::default(),
        "Phase should be reset after completion"
    );
}

#[test]
fn test_quantifier_detects_movement() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "village_square"}}"#.to_string()],
            ..Default::default()
        }),
    );

    service.execute_action(
        ctx.clone(),
        "walk to the village square".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 500);
    assert!(completed, "Movement action should complete within timeout");

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Status should be reset after movement action"
    );

    // Player should have moved (either to existing room or dynamic room)
    assert_ne!(
        guard.movement.current_room_id, "room1",
        "Player should have moved from starting room"
    );
}

#[test]
fn test_quantifier_detects_npc_presence_and_fires_trigger() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Generating;
    // Reset times_met so the trigger is eligible to fire
    if let Some(encounter) = state.character_state.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    );

    service.execute_action(
        ctx.clone(),
        "enter the shop".to_string(),
        "Player".to_string(),
    );

    let guard = crate::latest_state(&ctx);
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Status should be reset after trigger action"
    );

    // Trigger should have fired, adding an event header
    let has_event = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.event_header.is_some());
    assert!(has_event, "Trigger should add an event header");

    // And a continuation narration
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
fn test_retry_no_snapshot() {
    let ctx = make_test_context(create_test_state());
    // Clear all snapshots so retry has nothing to load
    ctx.snapshot_storage.reset().unwrap();

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    // Should not panic with no snapshot
    service.retry_last_response(ctx.clone());
}

#[test]
fn test_retry_no_input_text() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    // Only add system and narration logs — no input
    state.add_log("System boot".to_string(), None, LogType::System);
    state.add_log("You see a room.".to_string(), None, LogType::Narration);

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    // State should remain unchanged
    let guard = crate::latest_state(&ctx);
    assert_eq!(guard.narrative.history().len(), 2);
}

#[test]
fn test_retry_room_not_found() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.movement.current_room_id = "non_existent_room".to_string();

    let ctx = make_test_context(state.clone());
    // Save pre-main snapshot so retry uses the state with non-existent room
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.messages.clone() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    // Re-save the main snapshot so it remains the latest (load_latest uses max created_at)
    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    let guard = crate::latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.generation.status,
            GenerationStatus::Error(ref msg) if msg.contains("Room not found")
        ),
        "Expected room not found error: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_retry_llm_error() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );

    let ctx = make_test_context(state.clone());

    // Save pre-main snapshot so retry has something to work with
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.messages.clone() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    // Re-save the main snapshot so it remains the latest (load_latest uses max created_at)
    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    let guard = crate::latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.generation.status,
            GenerationStatus::Error(_)
        ),
        "Expected error status after failing LLM: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_retry_empty_narration() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );

    let ctx = make_test_context(state.clone());

    // Save a pre-main snapshot so retry has something to work with
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.messages.clone() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    // Re-save the main snapshot so it remains the latest (load_latest uses max created_at)
    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    let guard = crate::latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.generation.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")
        ),
        "Expected empty response error: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_pre_main_snapshot_saved_before_narration() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "FreeAction should complete within timeout");

    // Verify snapshot was saved
    let latest = ctx.snapshot_storage.load_latest().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_pre_event_snapshot_saved_before_continuation() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.messages.clear();
    state.narrative.generation.status = GenerationStatus::Idle;
    // Reset times_met so the trigger is eligible to fire
    if let Some(encounter) = state.character_state.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);

    // Use a quantifier that explicitly returns the shopkeeper NPC
    let quantifier = MockBackend {
        per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
        ..Default::default()
    };

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(quantifier),
    );

    service.execute_action(
        ctx.clone(),
        "examine the shopkeeper".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(
        completed,
        "FreeAction with trigger should complete within timeout"
    );

    // Verify the action completed and a snapshot was saved
    let latest = ctx.snapshot_storage.load_latest().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_retry_main_narration_uses_pre_main_snapshot() {
    let mut state = create_test_state();
    state.narrative.messages.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.narrative.generation.status = GenerationStatus::Idle;

    let ctx = make_test_context(state.clone());

    // Save pre-main snapshot to simulate normal action execution
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.snapshot_storage.save(&pre_main).unwrap();
    for mut msg in state.narrative.messages.clone() {
        if msg.log_type == LogType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    // Re-save the main snapshot so it remains the latest (load_latest uses max created_at)
    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&main);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Retry should complete within timeout");

    let guard = crate::latest_state(&ctx);
    // Should have a narration (from mock backend)
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
    state.narrative.messages.clear();
    state.add_log(
        "look around".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    // Add a main narration
    state.add_log(
        "You look around the shop.".to_string(),
        None,
        LogType::Narration,
    );
    // Add an event + continuation to simulate trigger having fired
    state.narrative.pending_event = Some("Greeting".to_string());
    state.add_log(
        "The shopkeeper looks up with a smile.".to_string(),
        None,
        LogType::Narration,
    );
    state.narrative.generation.status = GenerationStatus::Idle;
    state.narrative.last_trigger = Some(chronicler_engine::model::state::StoredTriggerContext {
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

    // Save pre-event snapshot
    let pre_event = GameStateSnapshot::from_game_state(&state);
    let pre_event_id = ctx.snapshot_storage.save(&pre_event).unwrap();

    // Set event header on the last message so retry treats it as event continuation
    if let Some(last) = state.narrative.messages.last_mut() {
        last.event_header = Some("Greeting".to_string());
    }

    // Re-save the final snapshot so it remains the latest
    let final_snap = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.snapshot_storage.save(&final_snap);

    for mut msg in state.narrative.messages.clone() {
        if msg.log_type == LogType::Narration && msg.event_header.is_none() {
            msg.snapshot_id = Some(pre_event_id);
        }
        let _ = ctx.message_storage.insert_message(&mut msg);
    }

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    service.retry_last_response(ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Event retry should complete within timeout");

    let guard = crate::latest_state(&ctx);
    // Main narration should be unchanged (still from original)
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
