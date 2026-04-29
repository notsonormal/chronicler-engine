//! Action processing logic extracted from fragments.rs
//!
//! This module contains pure functions for game action processing,
//! moved to a separate module for better testability and code coverage.

use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room, get_current_room};
use crate::engine::trigger_eval::{evaluate_triggers, mark_trigger_fired};
use crate::model::character::NpcCard;
use crate::model::state::{GameState, LogType};
use crate::narrative::prompt::{PhiMode, PromptBuilder, PromptContext};
use crate::narrative::quantifier::NpcEvent;

/// [DOC: docs/architecture/system.md]
pub fn get_static_npcs(state: &GameState, room_npc_ids: &[String]) -> Vec<NpcCard> {
    room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect()
}

/// [DOC: docs/architecture/system.md]
pub fn handle_movement(state: &mut GameState, destination: Option<&str>, new_npc_ids: &[String]) {
    let Some(trigger) = destination else {
        return;
    };

    let previous_room_id = state.current_room_id.clone();

    let success = match attempt_semantic_walk(state, trigger) {
        Ok(_) => true,
        Err(_) => {
            let dynamic_room = create_dynamic_room(trigger, "A place you have never seen before.");
            state
                .dynamic_rooms
                .insert(dynamic_room.id.clone(), dynamic_room.clone());
            state.current_room_id = dynamic_room.id.clone();
            true
        }
    };

    if !success {
        return;
    }

    if previous_room_id != state.current_room_id {
        for npc_id in new_npc_ids {
            // Set currently_meeting = true for NPCs in the new room.
            // NOTE: times_met is NOT incremented here - it's handled by the NPC event layer
            // in fragments.rs (compute_npc_events -> "Entered" events), which prevents
            // double-increment when an NPC enters a room.
            state.character_state.set_currently_meeting(npc_id, true);
        }
    }

    if let Ok(current_room) = get_current_room(state) {
        state.add_log(
            String::new(),
            Some(current_room.name.clone()),
            LogType::Narration,
        );
    }
}

/// [DOC: docs/architecture/system.md]
pub fn apply_npc_events(state: &mut GameState, events: &[NpcEvent]) {
    for event in events {
        match event.event_type {
            crate::narrative::quantifier::NpcEventType::Entered => {
                state
                    .character_state
                    .set_currently_meeting(&event.npc_id, true);
                state.character_state.increment_times_met(&event.npc_id);
            }
            crate::narrative::quantifier::NpcEventType::Left => {
                state
                    .character_state
                    .set_currently_meeting(&event.npc_id, false);
            }
        }
    }
}

/// [DOC: docs/system/triggers.md]
pub fn evaluate_and_narrate_triggers(
    state: &mut GameState,
    narration_text: &str,
    trigger_context: &PromptContext<'_>,
    max_triggers: usize,
) {
    let matching_triggers = evaluate_triggers(state);

    for (trigger_idx, (npc, trigger)) in matching_triggers.iter().take(max_triggers).enumerate() {
        let Ok(room) = get_current_room(state) else {
            continue;
        };

        let continuation_user_msg = format!(
            "Previous narration:\n{narration_text}\n\nTrigger event: {}",
            trigger.action.narration_prompt
        );

        let trigger_ctx = PromptContext {
            world: trigger_context.world,
            room,
            all_npcs: trigger_context.all_npcs,
            npcs_in_area: &state.npcs_in_area,
            player: trigger_context.player,
            user_message: &continuation_user_msg,
            history: &state.narration_history,
        };

        let mut pb = PromptBuilder::from_context(&trigger_ctx);
        pb.phi_mode = PhiMode::Continuation;

        let Ok((system_prompt, user_prompt)) = pb.build_split() else {
            log::error!(
                "Failed to build continuation prompt: {}",
                "build_split failed"
            );
            continue;
        };

        let backend = crate::narrative::llm::get_llm_backend();
        let continuation_text =
            match backend.narrate_action_from_prompt(&system_prompt, &user_prompt) {
                Ok(text) => text,
                Err(e) => {
                    log::error!("Trigger narration failed: {e}");
                    state.add_log(
                        format!("[Trigger narration failed: {e}]"),
                        None,
                        LogType::System,
                    );
                    continue;
                }
            };

        if continuation_text.trim().is_empty() {
            continue;
        }
        state.add_log(continuation_text, None, LogType::Narration);
        if !trigger.repeat {
            mark_trigger_fired(state, &npc.id, trigger_idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Create a minimal GameState for testing.
    fn make_test_state() -> GameState {
        let world = Arc::new(crate::model::world::WorldCard {
            name: "Test World".to_string(),
            description: "A test world".to_string(),
            global_rules: vec![],
            default_room_image: None,
        });

        let room = crate::model::map::Room {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            description: "A test room".to_string(),
            exits: std::collections::HashMap::new(),
            items: vec![],
            npcs: vec!["carla".to_string()],
            image_path: None,
            navigation_description: None,
        };

        let region = crate::model::map::Region {
            id: "test".to_string(),
            name: "Test".to_string(),
            rooms: vec![room],
        };

        let overworld = crate::model::map::Overworld {
            id: "test".to_string(),
            name: "Test".to_string(),
            regions: vec![region],
        };

        let map = Arc::new(crate::model::map::MapDef { overworld });

        let npc = NpcCard {
            id: "carla".to_string(),
            sheet: crate::model::character::CharacterSheet {
                name: "Carla".to_string(),
                description: "A test NPC".to_string(),
                personality: "Friendly".to_string(),
                scenario: "Test scenario".to_string(),
                example_dialogue: "Hello!".to_string(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };

        let mut npcs = std::collections::HashMap::new();
        npcs.insert("carla".to_string(), npc);

        let player = Arc::new(crate::model::character::PlayerCard {
            sheet: crate::model::character::CharacterSheet {
                name: "Player".to_string(),
                description: "The player".to_string(),
                personality: "Brave".to_string(),
                scenario: "Test scenario".to_string(),
                example_dialogue: "Hello!".to_string(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        });

        GameState {
            world,
            map,
            player,
            npcs,
            current_room_id: "test_room".to_string(),
            narration_history: vec![],
            next_log_id: 1,
            npcs_in_area: vec![],
            dynamic_rooms: std::collections::HashMap::new(),
            character_state: Default::default(),
            generation_state: Default::default(),
        }
    }

    #[test]
    fn test_get_static_npcs_returns_npcs() {
        let state = make_test_state();
        let room_npcs = vec!["carla".to_string()];
        let result = get_static_npcs(&state, &room_npcs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "carla");
    }

    #[test]
    fn test_get_static_npcs_empty_for_unknown() {
        let state = make_test_state();
        let room_npcs = vec!["unknown".to_string()];
        let result = get_static_npcs(&state, &room_npcs);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_npc_events_entered() {
        let mut state = make_test_state();
        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Entered,
        }];

        apply_npc_events(&mut state, &events);

        assert!(state.character_state.is_currently_meeting("carla"));
    }

    #[test]
    fn test_apply_npc_events_left() {
        let mut state = make_test_state();
        state.character_state.set_currently_meeting("carla", true);

        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Left,
        }];

        apply_npc_events(&mut state, &events);

        assert!(!state.character_state.is_currently_meeting("carla"));
    }

    #[test]
    fn test_apply_npc_events_increments_times_met() {
        let mut state = make_test_state();
        let initial_times = state.character_state.get_times_met("carla");

        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Entered,
        }];

        apply_npc_events(&mut state, &events);

        assert_eq!(
            state.character_state.get_times_met("carla"),
            initial_times + 1
        );
    }

    #[test]
    fn test_handle_movement_no_destination() {
        let mut state = make_test_state();
        let original_room = state.current_room_id.clone();

        handle_movement(&mut state, None, &["carla".to_string()]);

        // Room should not change when destination is None
        assert_eq!(state.current_room_id, original_room);
    }

    #[test]
    fn test_handle_movement_same_room_no_increment() {
        let mut state = make_test_state();
        // Already in test_room, moving to same room
        state.current_room_id = "test_room".to_string();
        let initial_times = state.character_state.get_times_met("carla");

        handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

        // times_met should not increment when room doesn't change
        assert_eq!(state.character_state.get_times_met("carla"), initial_times);
    }

    #[test]
    fn test_handle_movement_creates_dynamic_room() {
        let mut state = make_test_state();
        let original_room = state.current_room_id.clone();

        // Attempt to move to a non-existent room
        handle_movement(&mut state, Some("nonexistent_room"), &[]);

        // Should create a dynamic room
        assert_ne!(state.current_room_id, original_room);
        assert!(state.dynamic_rooms.contains_key(&state.current_room_id));
    }

    #[test]
    fn test_handle_movement_success_adds_room_log() {
        let mut state = make_test_state();

        // Move to existing room (test_room exists in the map)
        handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

        // Should have added a narration entry with room name
        assert!(!state.narration_history.is_empty());
        let last_entry = state.narration_history.last().unwrap();
        assert_eq!(last_entry.log_type, LogType::Narration);
        assert_eq!(last_entry.sender, Some("Test Room".to_string()));
    }

    #[test]
    fn test_handle_movement_sets_currently_meeting() {
        let mut state = make_test_state();

        // Move to a different room (creates dynamic room)
        handle_movement(&mut state, Some("new_room"), &["carla".to_string()]);

        // Should set currently_meeting for NPCs in new room
        assert!(state.character_state.is_currently_meeting("carla"));
    }
}
