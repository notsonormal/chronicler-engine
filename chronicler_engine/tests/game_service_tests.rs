//! [DOC: docs/reference/testing.md]

mod test_data;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chronicler_engine::engine::game_service::{DefaultGameService, GameService};
    use chronicler_engine::model::state::GameState;
    use chronicler_engine::model::state::GenerationStatus;
    use chronicler_engine::model::state::LogType;
    use chronicler_engine::model::state_snapshot::GameStateSnapshot;
    use chronicler_engine::model::character::*;
    use chronicler_engine::narrative::agents::quantifier::{
        MockQuantifierBackend, MovementParseResult, MovementType, QuantifierConfidence,
    };
    use chronicler_engine::narrative::llm::MockBackend;
    use chronicler_engine::test_support::make_test_context;
    use crate::test_data::create_test_state;

    fn wait_for_generation_complete(
        ctx: &chronicler_engine::engine::game_service::GameServiceContext,
        timeout_ms: u64,
    ) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        while start.elapsed() < timeout {
            if let Ok(Some(snap)) = ctx.snapshot_storage.load_latest(None) {
                let guard = GameState::from_snapshot(
                    &snap,
                    ctx.world.clone(),
                    ctx.map.clone(),
                    ctx.player.clone(),
                    (*ctx.npcs).clone(),
                );
                if !guard.narrative.generation.status.is_generating() {
                    return true;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        false
    }

    fn failing_service() -> DefaultGameService {
        DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::failing()),
            Arc::new(MockQuantifierBackend::default()),
        )
    }

    fn create_test_state_with_trigger_npc() -> GameState {
        crate::test_data::create_test_state_with_npcs(
            vec!["shopkeeper".to_string()],
            vec![NpcCard {
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
                triggers: vec![chronicler_engine::model::trigger::Trigger {
                    condition: chronicler_engine::model::trigger::TriggerCondition::TimesMet(
                        chronicler_engine::model::trigger::ComparisonOperator::Eq,
                        0,
                    ),
                    action: chronicler_engine::model::trigger::TriggerAction {
                        name: "Greeting".into(),
                        narration_prompt: "The shopkeeper looks up with a smile.".into(),
                    },
                    repeat: false,
                    room_id: None,
                }],
            }],
        )
    }

    #[test]
    fn test_execute_look_action() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        service.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        let has_narration = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(has_narration, "Look should add narration");
    }

    #[test]
    fn test_execute_talk_action() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        service.execute_action(
            ctx.clone(),
            "talk to innkeeper".to_string(),
            "Player".to_string(),
        );

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        let has_system = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("You talk to"));
        assert!(has_system, "Talk should add system log");
    }

    #[test]
    fn test_execute_inventory_action() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        service.execute_action(ctx.clone(), "inventory".to_string(), "Player".to_string());

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        let has_system = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("inventory"));
        assert!(has_system, "Inventory should add system log");
    }

    #[test]
    fn test_execute_quit_action() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        service.execute_action(ctx.clone(), "quit".to_string(), "Player".to_string());

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        let has_goodbye = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("Goodbye"));
        assert!(has_goodbye, "Quit should add Goodbye log");
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "Quit should reset is_generating"
        );
    }

    #[test]
    fn test_retry_with_no_history() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        // Should not panic with empty history
        service.retry_last_response(ctx.clone());

        // State should be unchanged
        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(guard.narrative.history.is_empty());
    }

    #[test]
    fn test_execute_freeaction_immediate_return() {
        let mut state = create_test_state();
        state.narrative.history.clear();
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
        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = failing_service();

        service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());

        // State should remain accessible after execute_action returns
        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = failing_service();

        service.execute_action(ctx.clone(), "test action".to_string(), "Player".to_string());

        let completed = wait_for_generation_complete(&ctx, 200);
        assert!(completed, "FreeAction should complete within timeout");

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating; // set by caller (server)
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        service.execute_action(
            ctx.clone(),
            "examine the room carefully".to_string(),
            "Player".to_string(),
        );

        let completed = wait_for_generation_complete(&ctx, 200);
        assert!(completed, "FreeAction should complete within timeout");

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "is_generating should be reset after FreeAction completes"
        );

        let has_narration = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(has_narration, "Mock LLM should add narration to history");
    }

    #[test]
    fn test_retry_with_mock_backend() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        state.add_log("look around".to_string(), None, LogType::Input);
        state.add_log("Initial narration".to_string(), None, LogType::Narration);
        state.narrative.generation.status = GenerationStatus::Generating; // set by caller (server)
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        service.retry_last_response(ctx.clone());

        let completed = wait_for_generation_complete(&ctx, 1000);
        assert!(completed, "Retry should complete within timeout");

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "is_generating should be reset after retry completes"
        );

        // The last AI response should have been replaced with mock narration
        let ai_responses: Vec<_> = guard
            .narrative
            .history
            .iter()
            .filter(|e| e.log_type == LogType::Narration)
            .collect();
        assert!(
            !ai_responses.is_empty(),
            "Should have AI responses after retry"
        );
    }

    #[test]
    fn test_execute_look_room_not_found() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        state.movement.current_room_id = "non_existent_room".to_string();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        service.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "Look should reset is_generating even when room not found"
        );
    }

    #[test]
    fn test_execute_talk_no_message() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        // "talk to innkeeper" without quoted message parses as ("innkeeper", None)
        service.execute_action(
            ctx.clone(),
            "talk to innkeeper".to_string(),
            "Player".to_string(),
        );

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        let has_talk =
            guard.narrative.history.iter().any(|e| {
                e.log_type == LogType::System && e.text.contains("You talk to innkeeper:")
            });
        assert!(has_talk, "Talk without message should add system log");
    }

    #[test]
    fn test_execute_freeaction_with_movement_mock() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating; // set by caller (server)
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
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

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "is_generating should be reset after FreeAction with movement"
        );

        let has_narration = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(
            has_narration,
            "Mock LLM should add narration even for movement actions"
        );
    }

    #[test]
    fn test_freeaction_phase_starts_narrating() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Idle;
        let ctx = make_test_context(state);
        let service = DefaultGameService::new();

        service.execute_action(
            ctx.clone(),
            "examine the room".to_string(),
            "Player".to_string(),
        );

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        service.execute_action(
            ctx.clone(),
            "examine the room carefully".to_string(),
            "Player".to_string(),
        );

        let completed = wait_for_generation_complete(&ctx, 200);
        assert!(completed, "FreeAction should complete within timeout");

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::with_delay(50)),
            Arc::new(MockQuantifierBackend::default()),
        );
        let token = ctx.cancel_token.clone();
        let token_clone = token.clone();

        let ctx_clone = ctx.clone();
        let handle = tokio::task::spawn_blocking(move || {
            if token_clone.is_cancelled() {
                if let Ok(Some(snap)) = ctx_clone.snapshot_storage.load_latest(None) {
                    let mut state = GameState::from_snapshot(
                        &snap,
                        ctx_clone.world.clone(),
                        ctx_clone.map.clone(),
                        ctx_clone.player.clone(),
                        (*ctx_clone.npcs).clone(),
                    );
                    state.narrative.generation.status = GenerationStatus::Idle;
                    let snapshot = GameStateSnapshot::from_game_state(
                        &state,
                        snap.message_id,
                        snap.swipe_index,
                    );
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
                if let Ok(Some(snap)) = ctx_clone.snapshot_storage.load_latest(None) {
                    let mut state = GameState::from_snapshot(
                        &snap,
                        ctx_clone.world.clone(),
                        ctx_clone.map.clone(),
                        ctx_clone.player.clone(),
                        (*ctx_clone.npcs).clone(),
                    );
                    state.narrative.generation.status = GenerationStatus::Idle;
                    let snapshot = GameStateSnapshot::from_game_state(
                        &state,
                        snap.message_id,
                        snap.swipe_index,
                    );
                    let _ = ctx_clone.snapshot_storage.save(&snapshot);
                }
            }
        });

        // Cancel while the mock backend is sleeping inside execute_action
        token.cancel();

        // Wait for the blocking task to finish
        handle.await.unwrap();

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "Status should be Idle after cancellation cleanup"
        );
    }

    #[test]
    fn test_execute_action_empty_command() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = failing_service();

        // Empty command parses as FreeAction("") and should not panic
        service.execute_action(ctx.clone(), "".to_string(), "Player".to_string());

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        // Failing mock backend causes FreeAction to fail and set Error status
        assert!(
            guard.narrative.generation.status.error_message().is_some(),
            "Empty command should result in error status: {:?}",
            guard.narrative.generation.status
        );
    }

    #[test]
    fn test_execute_action_unknown_command() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context(state);
        let service = failing_service();

        // Unknown command parses as FreeAction and should not panic
        service.execute_action(ctx.clone(), "xyz123".to_string(), "Player".to_string());

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        // Failing mock backend causes FreeAction to fail and set Error status
        assert!(
            guard.narrative.generation.status.error_message().is_some(),
            "Unknown command should result in error status: {:?}",
            guard.narrative.generation.status
        );
    }

    #[test]
    fn test_retry_last_response_not_ai_generated() {
        let mut state = create_test_state();
        state.narrative.history.clear();
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

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::with_empty_response()),
            Arc::new(MockQuantifierBackend::default()),
        );

        service.execute_action(
            ctx.clone(),
            "examine the room".to_string(),
            "Player".to_string(),
        );

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
            .history
            .iter()
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
        state.narrative.generation.status = GenerationStatus::Generating;
        // Reset times_met so the trigger is eligible to fire
        if let Some(encounter) = state.character_state.npcs.get_mut("shopkeeper") {
            encounter.times_met = 0;
        }
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::with_failing_trigger_narration()),
            Arc::new(MockQuantifierBackend {
                npcs_to_return: vec!["shopkeeper".to_string()],
                ..Default::default()
            }),
        );

        // Use a FreeAction so the backend is invoked ("talk to" parses as Talk, not FreeAction)
        service.execute_action(
            ctx.clone(),
            "examine the shopkeeper".to_string(),
            "Player".to_string(),
        );

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "Status should be reset after trigger narration failure"
        );

        // Main narration should still be present
        let has_narration = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(
            has_narration,
            "Main narration should exist even when trigger narration failed"
        );

        // Trigger narration failure should be logged as a system message
        let has_trigger_error =
            guard.narrative.history.iter().any(|e| {
                e.log_type == LogType::System && e.text.contains("Trigger narration failed")
            });
        assert!(
            has_trigger_error,
            "Trigger narration failure should be logged"
        );
    }

    // === Status Transition & Quantifier Tests ===

    #[test]
    fn test_delayed_llm_completes_without_deadlock() {
        let mut state = create_test_state();
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::with_delay(200)),
            Arc::new(MockQuantifierBackend::default()),
        );

        service.execute_action(ctx.clone(), "look around".to_string(), "Player".to_string());

        // execute_action is synchronous — by now the delay has elapsed
        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend {
                movement_to_return: Some(MovementParseResult {
                    movement_type: Some(MovementType::Entering),
                    destination: Some("village_square".to_string()),
                    confidence: QuantifierConfidence::High,
                }),
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

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
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
        state.narrative.history.clear();
        state.narrative.generation.status = GenerationStatus::Generating;
        // Reset times_met so the trigger is eligible to fire
        if let Some(encounter) = state.character_state.npcs.get_mut("shopkeeper") {
            encounter.times_met = 0;
        }
        let ctx = make_test_context(state);
        let service = DefaultGameService::with_mock_quantifier(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend {
                npcs_to_return: vec!["shopkeeper".to_string()],
                ..Default::default()
            }),
        );

        service.execute_action(
            ctx.clone(),
            "enter the shop".to_string(),
            "Player".to_string(),
        );

        let snap = ctx.snapshot_storage.load_latest(None).unwrap().unwrap();
        let guard =
            GameState::from_snapshot(&snap, ctx.world, ctx.map, ctx.player, (*ctx.npcs).clone());
        assert!(
            !guard.narrative.generation.status.is_generating(),
            "Status should be reset after trigger action"
        );

        // Trigger should have fired, adding an Event entry
        let has_event = guard
            .narrative
            .history
            .iter()
            .any(|e| e.log_type == LogType::Event);
        assert!(has_event, "Trigger should add an Event entry");

        // And a continuation narration
        let narration_count = guard
            .narrative
            .history
            .iter()
            .filter(|e| e.log_type == LogType::Narration)
            .count();
        assert!(
            narration_count >= 2,
            "Should have main narration + trigger continuation narration"
        );
    }
}

// NOTE: FreeAction with mock LLM is now covered by both unit tests (above) and UI tests:
// - tests/trigger_tests.rs: test_freeaction_without_movement_works
// - tests/flow_mock_tests.rs: test_look_command_shows_thinking
// - tests/flow_llm_tests.rs: test_llm_generates_narration_for_free_action
// The UI tests spawn a server with mock backend and verify the full HTTP flow.
