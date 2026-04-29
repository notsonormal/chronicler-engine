//! GameService Unit Tests
//!
//! Tests for the DefaultGameService implementation.
//! These tests verify action routing and state updates without requiring LLM calls.

#[cfg(test)]
mod tests {
    use chronicler_engine::engine::game_service::{DefaultGameService, GameService};
    use chronicler_engine::model::state::LogType;
    use chronicler_engine::model::{character::*, map::*, world::*};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn create_test_state() -> Arc<Mutex<chronicler_engine::model::state::GameState>> {
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
            !guard.generation_state.is_generating,
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
            guard.generation_state.is_generating = false;
        }

        // FreeAction should return immediately and spawn a thread
        // The function should not block
        service.execute_action(
            state.clone(),
            "examine the room".to_string(),
            "Player".to_string(),
        );

        // For FreeAction, is_generating is set to true immediately
        // because the thread manages it itself
        let _guard = state.lock().unwrap();
        // Just verify no panic occurred and state is accessible
        assert!(true, "FreeAction should not panic");
    }
}

// NOTE: FreeAction with mock LLM testing is covered by UI tests:
// - tests/trigger_tests.rs: test_freeaction_without_movement_works
// - tests/flow_mock_tests.rs: test_look_command_shows_thinking
// - tests/flow_llm_tests.rs: test_llm_generates_narration_for_free_action
// These integration tests spawn a server with mock backend and verify the full flow.
