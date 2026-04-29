//! Logic Module Unit Tests
//!
//! Tests for navigation and room functions in logic.rs

mod test_data;

#[cfg(test)]
mod tests {
    use super::*;
    use chronicler_engine::engine::logic::*;
    use chronicler_engine::model::state::GameState;
    use std::sync::{Arc, Mutex};

    fn create_navigation_test_state() -> Arc<Mutex<GameState>> {
        let world = Arc::new(test_data::create_test_world());
        let map = Arc::new(test_data::create_navigation_test_map());
        let player = Arc::new(test_data::create_test_player());

        Arc::new(Mutex::new(GameState::new(
            world,
            map,
            player,
            vec![],
            "entrance".to_string(),
        )))
    }

    #[test]
    fn test_find_room_in_map_success() {
        let map = test_data::create_navigation_test_map();
        assert!(find_room_in_map(&map, "entrance").is_some());
        assert!(find_room_in_map(&map, "hall").is_some());
        assert!(find_room_in_map(&map, "kitchen").is_some());
    }

    #[test]
    fn test_find_room_in_map_failure() {
        let map = test_data::create_navigation_test_map();
        assert!(find_room_in_map(&map, "nonexistent").is_none());
        assert!(find_room_in_map(&map, "basement").is_none());
    }

    #[test]
    fn test_get_current_room() {
        let state = create_navigation_test_state();
        let guard = state.lock().unwrap();
        let room = get_current_room(&guard).unwrap();
        assert_eq!(room.id, "entrance");
        assert_eq!(room.name, "Mansion Entrance");
    }

    #[test]
    fn test_get_available_exits() {
        let state = create_navigation_test_state();
        let guard = state.lock().unwrap();
        let exits = get_available_exits(&guard);
        assert!(exits.contains(&"North".to_string()));
        assert_eq!(exits.len(), 1);
    }

    #[test]
    fn test_get_available_exits_multiple() {
        // Move to hall which has 3 exits
        {
            let state = create_navigation_test_state();
            let mut guard = state.lock().unwrap();
            guard.current_room_id = "hall".to_string();
        }

        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            guard.current_room_id = "hall".to_string();
        }

        let guard = state.lock().unwrap();
        let exits = get_available_exits(&guard);
        // Hall has South, East, West exits
        assert!(exits.contains(&"South".to_string()));
        assert!(exits.contains(&"East".to_string()));
        assert!(exits.contains(&"West".to_string()));
    }

    #[test]
    fn test_process_directional_movement_valid() {
        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            let result = process_directional_movement(&mut guard, "north");
            assert!(result.is_ok());
            assert_eq!(guard.current_room_id, "hall");
        }

        // Verify room changed
        let guard = state.lock().unwrap();
        assert_eq!(guard.current_room_id, "hall");
    }

    #[test]
    fn test_process_directional_movement_invalid() {
        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            // Entrance has only North, not West
            let result = process_directional_movement(&mut guard, "west");
            assert!(result.is_err());
            assert_eq!(guard.current_room_id, "entrance");
        }
    }

    #[test]
    fn test_process_directional_movement_by_room_name() {
        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            // Should work with room name too
            let result = process_directional_movement(&mut guard, "Main Hall");
            assert!(result.is_ok());
            assert_eq!(guard.current_room_id, "hall");
        }
    }

    #[test]
    fn test_attempt_semantic_walk_valid() {
        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            let result = attempt_semantic_walk(&mut guard, "kitchen");
            assert!(result.is_ok());
            assert!(result.unwrap().contains("Kitchen"));
            assert_eq!(guard.current_room_id, "kitchen");
        }
    }

    #[test]
    fn test_attempt_semantic_walk_invalid() {
        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            let result = attempt_semantic_walk(&mut guard, "nonexistent_room");
            assert!(result.is_err());
            // Room ID should not change
            assert_eq!(guard.current_room_id, "entrance");
        }
    }

    #[test]
    fn test_process_directional_movement_case_insensitive() {
        let state = create_navigation_test_state();
        {
            let mut guard = state.lock().unwrap();
            let result = process_directional_movement(&mut guard, "NORTH");
            assert!(result.is_ok());
            assert_eq!(guard.current_room_id, "hall");
        }
    }
}
