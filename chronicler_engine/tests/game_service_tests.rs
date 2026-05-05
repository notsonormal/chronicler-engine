//! [DOC: docs/reference/testing.md]

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chronicler_engine::engine::game_service::{DefaultGameService, GameService};
    use chronicler_engine::model::state::GenerationStatus;
    use chronicler_engine::model::state::LogType;
    use chronicler_engine::model::{character::*, map::*, world::*};
    use chronicler_engine::narrative::llm::MockBackend;
    use chronicler_engine::narrative::quantifier::MockQuantifierBackend;
    use tokio_util::sync::CancellationToken;

    /// Poll for the FreeAction/retry thread to complete by checking is_generating.
    /// Returns true if generation completed within timeout, false on timeout.
    fn wait_for_generation_complete(
        state: &Arc<Mutex<chronicler_engine::model::state::GameState>>,
        timeout_ms: u64,
    ) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        while start.elapsed() < timeout {
            if let Ok(guard) = state.lock() {
                if !guard.generation_state.status.is_generating() {
                    return true;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        false
    }

    pub fn create_test_state() -> Arc<Mutex<chronicler_engine::model::state::GameState>> {
        let world = Arc::new(WorldCard {
            name: "Test World".into(),
            description: "A test world".into(),
            global_rules: vec![],
            default_room_image: None,
        });

        let room1_exits = HashMap::new();
        let room1 = Room {
            id: "room1".into(),
            name: "Test Tavern".into(),
            description: "A cozy tavern with wooden beams and warm fire.".into(),
            exits: room1_exits,
            items: vec![],
            npcs: vec!["test_npc".to_string()],
            image_path: None,
            navigation_description: None,
        };

        let region = Region {
            id: "test_region".into(),
            name: "Test Region".into(),
            rooms: vec![room1],
        };

        let overworld = Overworld {
            id: "test_overworld".into(),
            name: "Test World".into(),
            regions: vec![region],
        };

        let map = Arc::new(MapDef { overworld });

        let player = Arc::new(PlayerCard {
            sheet: CharacterSheet {
                name: "Test Player".into(),
                description: "A test player".into(),
                personality: "Brave".into(),
                scenario: "Test scenario".into(),
                example_dialogue: "Hello!".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        });

        let npcs = vec![NpcCard {
            id: "test_npc".into(),
            sheet: CharacterSheet {
                name: "Innkeeper".into(),
                description: "A friendly innkeeper".into(),
                personality: "Helpful".into(),
                scenario: "Runs the tavern".into(),
                example_dialogue: "Welcome!".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        }];

        Arc::new(Mutex::new(chronicler_engine::model::state::GameState::new(
            world,
            map,
            player,
            npcs,
            "room1".to_string(),
        )))
    }

    #[test]
    fn test_execute_look_action() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        // Clear any existing history
        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        service.execute_action(state.clone(), "look".to_string(), "Player".to_string());

        let guard = state.lock().unwrap();
        let has_narration = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(has_narration, "Look should add narration");
    }

    #[test]
    fn test_execute_talk_action() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        service.execute_action(
            state.clone(),
            "talk to innkeeper".to_string(),
            "Player".to_string(),
        );

        let guard = state.lock().unwrap();
        let has_system = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("You talk to"));
        assert!(has_system, "Talk should add system log");
    }

    #[test]
    fn test_execute_inventory_action() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        service.execute_action(state.clone(), "inventory".to_string(), "Player".to_string());

        let guard = state.lock().unwrap();
        let has_system = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("inventory"));
        assert!(has_system, "Inventory should add system log");
    }

    #[test]
    fn test_execute_quit_action() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        service.execute_action(state.clone(), "quit".to_string(), "Player".to_string());

        let guard = state.lock().unwrap();
        let has_goodbye = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("Goodbye"));
        assert!(has_goodbye, "Quit should add Goodbye log");
        assert!(
            !guard.generation_state.status.is_generating(),
            "Quit should reset is_generating"
        );
    }

    #[test]
    fn test_retry_with_no_history() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        // Ensure history is empty
        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        // Should not panic with empty history
        service.retry_last_response(state.clone());

        // State should be unchanged
        let guard = state.lock().unwrap();
        assert!(guard.narration_history.is_empty());
    }

    #[test]
    fn test_execute_freeaction_immediate_return() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status = chronicler_engine::model::state::GenerationStatus::Idle;
        }

        // FreeAction should return immediately and spawn a thread
        // The function should not block
        service.execute_action(
            state.clone(),
            "examine the room".to_string(),
            "Player".to_string(),
        );

        // State should be accessible immediately after execute_action returns
        // (the thread runs in background)
        let guard = state.lock().unwrap();
        let status = &guard.generation_state.status;
        // DefaultGameService has no API key, so FreeAction fails and sets Error status
        assert!(
            status.error_message().is_some(),
            "Status should be Error after failed FreeAction: {status:?}"
        );
    }

    /// Test that FreeAction handles room-not-found gracefully.
    /// When current_room_id points to a non-existent room, the thread
    /// should reset is_generating and exit without panicking.
    #[test]
    fn test_execute_freeaction_room_not_found() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        // Set current_room_id to a room that doesn't exist
        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating;
            guard.current_room_id = "non_existent_room".to_string();
        }

        // Execute FreeAction - should not panic
        service.execute_action(
            state.clone(),
            "examine the room".to_string(),
            "Player".to_string(),
        );

        // Verify is_generating was reset (room not found path)
        let completed = wait_for_generation_complete(&state, 1000);
        assert!(
            completed,
            "is_generating should be reset when room not found"
        );
    }

    /// Test that FreeAction returns without blocking and state remains accessible.
    #[test]
    fn test_execute_freeaction_state_accessible() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating;
        }

        service.execute_action(
            state.clone(),
            "look around".to_string(),
            "Player".to_string(),
        );

        // State should remain accessible after execute_action returns
        let guard = state.lock().unwrap();
        let status = &guard.generation_state.status;
        // DefaultGameService has no API key, so FreeAction fails and sets Error status
        assert!(
            status.error_message().is_some(),
            "Status should be Error after failed FreeAction: {status:?}"
        );
    }

    /// Test that FreeAction with LLM backend narration failure
    /// sets error_message and resets is_generating.
    #[test]
    fn test_execute_freeaction_narration_failure() {
        let state = create_test_state();
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::failing()),
            Arc::new(MockQuantifierBackend::default()),
        );

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating;
        }

        service.execute_action(
            state.clone(),
            "test action".to_string(),
            "Player".to_string(),
        );

        let completed = wait_for_generation_complete(&state, 200);
        assert!(completed, "FreeAction should complete within timeout");

        let guard = state.lock().unwrap();
        // MockBackend::failing() always returns an error
        assert!(
            guard.generation_state.status.error_message().is_some(),
            "Should have error after failed narration: {:?}",
            guard.generation_state.status
        );
    }

    /// Test successful FreeAction flow with mock LLM backend.
    /// Verifies that narration is added and is_generating is properly reset.
    #[test]
    fn test_execute_freeaction_with_mock_backend() {
        let state = create_test_state();
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating; // set by caller (server)
        }

        service.execute_action(
            state.clone(),
            "examine the room carefully".to_string(),
            "Player".to_string(),
        );

        let completed = wait_for_generation_complete(&state, 200);
        assert!(completed, "FreeAction should complete within timeout");

        let guard = state.lock().unwrap();
        assert!(
            !guard.generation_state.status.is_generating(),
            "is_generating should be reset after FreeAction completes"
        );

        let has_narration = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(has_narration, "Mock LLM should add narration to history");
    }

    /// Test retry_last_response with mock backend and existing history.
    /// Verifies that the retry path works end-to-end.
    #[test]
    fn test_retry_with_mock_backend() {
        let state = create_test_state();
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        // Set up history with a player input and AI response
        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.add_log("look around".to_string(), None, LogType::Input);
            guard.add_log("Initial narration".to_string(), None, LogType::Narration);
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating; // set by caller (server)
        }

        service.retry_last_response(state.clone());

        let completed = wait_for_generation_complete(&state, 1000);
        assert!(completed, "Retry should complete within timeout");

        let guard = state.lock().unwrap();
        assert!(
            !guard.generation_state.status.is_generating(),
            "is_generating should be reset after retry completes"
        );

        // The last AI response should have been replaced with mock narration
        let ai_responses: Vec<_> = guard
            .narration_history
            .iter()
            .filter(|e| e.log_type == LogType::Narration)
            .collect();
        assert!(
            !ai_responses.is_empty(),
            "Should have AI responses after retry"
        );
    }

    /// Test Look action when get_current_room fails.
    /// Verifies no panic and is_generating is reset.
    #[test]
    fn test_execute_look_room_not_found() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.current_room_id = "non_existent_room".to_string();
        }

        service.execute_action(state.clone(), "look".to_string(), "Player".to_string());

        let guard = state.lock().unwrap();
        assert!(
            !guard.generation_state.status.is_generating(),
            "Look should reset is_generating even when room not found"
        );
    }

    /// Test Talk action without quoted message (msg = None).
    #[test]
    fn test_execute_talk_no_message() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        // "talk to innkeeper" without quoted message parses as ("innkeeper", None)
        service.execute_action(
            state.clone(),
            "talk to innkeeper".to_string(),
            "Player".to_string(),
        );

        let guard = state.lock().unwrap();
        let has_talk = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::System && e.text.contains("You talk to innkeeper:"));
        assert!(has_talk, "Talk without message should add system log");
    }

    /// Test FreeAction with movement using mock backend.
    /// Verifies that the quantifier runs and NPC detection works.
    #[test]
    fn test_execute_freeaction_with_movement_mock() {
        let state = create_test_state();
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating; // set by caller (server)
        }

        // Action that implies movement
        service.execute_action(
            state.clone(),
            "walk to the north".to_string(),
            "Player".to_string(),
        );

        let completed = wait_for_generation_complete(&state, 1000);
        assert!(
            completed,
            "FreeAction with movement should complete within timeout"
        );

        let guard = state.lock().unwrap();
        assert!(
            !guard.generation_state.status.is_generating(),
            "is_generating should be reset after FreeAction with movement"
        );

        let has_narration = guard
            .narration_history
            .iter()
            .any(|e| e.log_type == LogType::Narration);
        assert!(
            has_narration,
            "Mock LLM should add narration even for movement actions"
        );
    }

    /// Test that execute_action handles poisoned initial mutex gracefully.
    #[test]
    fn test_execute_action_poisoned_mutex() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        // Poison the mutex by panicking while holding the lock
        let _ = std::panic::catch_unwind(|| {
            let _guard = state.lock().unwrap();
            panic!("intentional panic to poison mutex");
        });

        // Should return early without panicking
        service.execute_action(state.clone(), "look".to_string(), "Player".to_string());

        // If we get here, the function handled poisoned mutex gracefully
        let guard = state.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            guard.narration_history.is_empty(),
            "Poisoned mutex should cause early return with no history changes"
        );
    }

    #[test]
    fn test_freeaction_phase_starts_narrating() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status = chronicler_engine::model::state::GenerationStatus::Idle;
        }

        service.execute_action(
            state.clone(),
            "examine the room".to_string(),
            "Player".to_string(),
        );

        let guard = state.lock().unwrap();
        // DefaultGameService has no API key, so FreeAction fails.
        // set_phase(Narrating) runs before the backend call, and set_error_and_reset
        // only updates status (not phase), so phase should still be Narrating.
        assert_eq!(
            guard.generation_state.phase,
            chronicler_engine::model::state::GenerationPhase::Narrating,
            "Phase should be Narrating after starting FreeAction: {:?}",
            guard.generation_state.status
        );
    }

    #[test]
    fn test_freeaction_phase_transitions_mock() {
        let state = create_test_state();
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            Arc::new(MockQuantifierBackend::default()),
        );

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status =
                chronicler_engine::model::state::GenerationStatus::Generating;
        }

        service.execute_action(
            state.clone(),
            "examine the room carefully".to_string(),
            "Player".to_string(),
        );

        let completed = wait_for_generation_complete(&state, 200);
        assert!(completed, "FreeAction should complete within timeout");

        let guard = state.lock().unwrap();
        assert!(
            !guard.generation_state.status.is_generating(),
            "Status should be reset after FreeAction completes"
        );
        assert_eq!(
            guard.generation_state.phase,
            chronicler_engine::model::state::GenerationPhase::default(),
            "Phase should be reset to default after completion"
        );
    }

    /// Test that cancellation token resets state to Idle after a task completes.
    /// Simulates the fragments.rs handler pattern: spawn_blocking with token checks.
    #[tokio::test]
    async fn test_cancellation_resets_state_to_idle() {
        let state = create_test_state();
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::with_delay(50)),
            Arc::new(MockQuantifierBackend::default()),
        );
        let token = CancellationToken::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.generation_state.status = GenerationStatus::Generating;
        }

        let state_clone = state.clone();
        let token_clone = token.clone();
        let handle = tokio::task::spawn_blocking(move || {
            if token_clone.is_cancelled() {
                if let Ok(mut guard) = state_clone.lock() {
                    guard.generation_state.status = GenerationStatus::Idle;
                }
                return;
            }
            service.execute_action(
                state_clone.clone(),
                "look around".to_string(),
                "Player".to_string(),
            );
            if token_clone.is_cancelled() {
                if let Ok(mut guard) = state_clone.lock() {
                    guard.generation_state.status = GenerationStatus::Idle;
                }
            }
        });

        // Cancel while the mock backend is sleeping inside execute_action
        token.cancel();

        // Wait for the blocking task to finish
        handle.await.unwrap();

        let guard = state.lock().unwrap();
        assert!(
            !guard.generation_state.status.is_generating(),
            "Status should be Idle after cancellation cleanup"
        );
    }

    #[test]
    fn test_execute_action_empty_command() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        // Empty command parses as FreeAction("") and should not panic
        service.execute_action(state.clone(), "".to_string(), "Player".to_string());

        let guard = state.lock().unwrap();
        // DefaultGameService has no API key, so it fails and sets Error status
        assert!(
            guard.generation_state.status.error_message().is_some(),
            "Empty command should result in error status: {:?}",
            guard.generation_state.status
        );
    }

    #[test]
    fn test_execute_action_unknown_command() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
        }

        // Unknown command parses as FreeAction and should not panic
        service.execute_action(state.clone(), "xyz123".to_string(), "Player".to_string());

        let guard = state.lock().unwrap();
        // DefaultGameService has no API key, so it fails and sets Error status
        assert!(
            guard.generation_state.status.error_message().is_some(),
            "Unknown command should result in error status: {:?}",
            guard.generation_state.status
        );
    }

    #[test]
    fn test_retry_last_response_not_ai_generated() {
        let state = create_test_state();
        let service = DefaultGameService::new();

        // Set up history with only an Input entry (no AI Narration after it)
        {
            let mut guard = state.lock().unwrap();
            guard.narration_history.clear();
            guard.add_log(
                "look around".to_string(),
                Some("Player".to_string()),
                LogType::Input,
            );
            guard.add_log("System message".to_string(), None, LogType::System);
        }

        // Retry should find the last input and attempt to process it
        // With DefaultGameService (no API key), it will fail
        service.retry_last_response(state.clone());

        // Wait for the retry to complete
        let completed = wait_for_generation_complete(&state, 1000);
        assert!(completed, "Retry should complete within timeout");

        let guard = state.lock().unwrap();
        assert!(
            guard.generation_state.status.error_message().is_some()
                || !guard.generation_state.status.is_generating(),
            "Retry with no AI response should complete: {:?}",
            guard.generation_state.status
        );
    }
}

// NOTE: FreeAction with mock LLM is now covered by both unit tests (above) and UI tests:
// - tests/trigger_tests.rs: test_freeaction_without_movement_works
// - tests/flow_mock_tests.rs: test_look_command_shows_thinking
// - tests/flow_llm_tests.rs: test_llm_generates_narration_for_free_action
// The UI tests spawn a server with mock backend and verify the full HTTP flow.
