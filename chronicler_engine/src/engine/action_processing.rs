//! [DOC: docs/architecture/system.md]

use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room, get_current_room};
use crate::engine::trigger_eval::{
    evaluate_triggers, increment_times_met, mark_trigger_fired, set_currently_meeting,
};
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::map::MapDef;
use crate::model::state::{GameState, LogType};
use crate::model::world::WorldCard;
use crate::narrative::prompt::{PhiMode, PromptBuilder, PromptContext};
use crate::narrative::quantifier::{NpcEvent, QuantifierResult, compute_npc_events};

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
            set_currently_meeting(&mut state.character_state, npc_id, true);
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
                set_currently_meeting(&mut state.character_state, &event.npc_id, true);
                increment_times_met(&mut state.character_state, &event.npc_id);
            }
            crate::narrative::quantifier::NpcEventType::Left => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, false);
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
            mark_trigger_fired(&mut state.character_state, &npc.id, trigger_idx);
        }
    }
}

/// Processes the result of a FreeAction LLM call.
///
/// This is the synchronous, testable core of FreeAction processing.
/// The LLM call and quantifier pass happen in the caller (game_service.rs thread).
///
/// # Arguments
/// * `state` - Game state to mutate
/// * `narration_text` - LLM-generated narration text
/// * `text` - Original user input text
/// * `quantifier_result` - Pre-computed quantifier result (from determine_npcs_in_room)
/// * `world` - World card reference
/// * `map` - Map definition reference
/// * `player` - Player card reference
/// * `all_npcs` - All NPCs in the game
/// * `room_npc_ids` - NPC IDs from current room's static config
/// * `history` - Narration history for trigger context
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(EngineError::RoomNotFound)` if current room not found
#[allow(clippy::too_many_arguments)]
pub fn execute_freeaction_impl(
    state: &mut GameState,
    narration_text: &str,
    text: &str,
    quantifier_result: &QuantifierResult,
    world: &WorldCard,
    _map: &MapDef,
    player: &crate::model::character::PlayerCard,
    all_npcs: &[NpcCard],
    _room_npc_ids: &[String],
    history: &[crate::model::state::LogEntry],
) -> Result<(), EngineError> {
    let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
    let previous_npc_ids: Vec<String> = previous_room_npcs.iter().map(|n| n.id.clone()).collect();

    // Handle movement if quantifier detected it
    handle_movement(
        state,
        quantifier_result.movement.destination.as_deref(),
        &quantifier_result.npcs.npc_ids,
    );

    // Build current NPCs from quantifier result
    let current_npcs: Vec<NpcCard> = quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids: Vec<String> = current_npcs.iter().map(|n| n.id.clone()).collect();

    // Get room data — clone to avoid holding &state borrow across mutating calls
    let room_data = get_current_room(state)
        .map_err(|_| EngineError::RoomNotFound("current room not found".to_string()))?
        .clone();

    // Now state is no longer borrowed — we can safely mutate it
    state.add_log(narration_text.to_string(), None, LogType::Narration);
    state.npcs_in_area = current_npcs.clone();

    // Build trigger context with owned room data
    let trigger_context = PromptContext {
        world,
        room: &room_data,
        all_npcs,
        npcs_in_area: &current_npcs,
        player,
        user_message: text,
        history,
    };

    // Evaluate triggers BEFORE incrementing times_met
    // (so TimesMet Eq 0 can fire on first detection)
    evaluate_and_narrate_triggers(state, narration_text, &trigger_context, 3);

    // Apply NPC events using the reusable function
    let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
    apply_npc_events(state, &events.events);

    Ok(())
}

// =============================================================================
// TESTS for execute_freeaction_impl
// =============================================================================

#[cfg(test)]
mod execute_freeaction_impl_tests {
    use super::*;
    use crate::model::character::{CharacterSheet, PlayerCard};
    use crate::model::map::{MapDef, Overworld, Region, Room};
    use crate::model::state::LogType;
    use crate::model::world::WorldCard;
    use crate::narrative::quantifier::{
        MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult,
    };
    use std::sync::Arc;

    fn make_test_world() -> Arc<WorldCard> {
        Arc::new(WorldCard {
            name: "Test World".to_string(),
            description: "A test world".to_string(),
            global_rules: vec![],
            default_room_image: None,
        })
    }

    fn make_test_map() -> Arc<MapDef> {
        let room = Room {
            id: "room1".to_string(),
            name: "Test Room".to_string(),
            description: "A test room".to_string(),
            exits: std::collections::HashMap::new(),
            items: vec![],
            npcs: vec!["carla".to_string()],
            image_path: None,
            navigation_description: None,
        };
        let region = Region {
            id: "region1".to_string(),
            name: "Test Region".to_string(),
            rooms: vec![room],
        };
        let overworld = Overworld {
            id: "overworld1".to_string(),
            name: "Test Overworld".to_string(),
            regions: vec![region],
        };
        Arc::new(MapDef { overworld })
    }

    fn make_test_player() -> Arc<PlayerCard> {
        Arc::new(PlayerCard {
            sheet: CharacterSheet {
                name: "Player".to_string(),
                description: "A test player".to_string(),
                personality: "Brave".to_string(),
                scenario: "Test".to_string(),
                example_dialogue: "Hello!".to_string(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        })
    }

    fn make_test_npcs() -> Vec<NpcCard> {
        vec![NpcCard {
            id: "carla".to_string(),
            sheet: CharacterSheet {
                name: "Carla".to_string(),
                description: "A friendly NPC".to_string(),
                personality: "Friendly".to_string(),
                scenario: "Test scenario".to_string(),
                example_dialogue: "Hello!".to_string(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        }]
    }

    fn make_test_state() -> GameState {
        let world = make_test_world();
        let map = make_test_map();
        let player = make_test_player();
        let npc = NpcCard {
            id: "carla".to_string(),
            sheet: CharacterSheet {
                name: "Carla".to_string(),
                description: "A friendly NPC".to_string(),
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

        GameState {
            world,
            map,
            player,
            npcs,
            current_room_id: "room1".to_string(),
            narration_history: vec![],
            next_log_id: 1,
            npcs_in_area: vec![],
            dynamic_rooms: std::collections::HashMap::new(),
            character_state: Default::default(),
            generation_state: Default::default(),
        }
    }

    fn make_quantifier_result_no_movement() -> QuantifierResult {
        QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["carla".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult {
                movement_type: None,
                destination: None,
                confidence: QuantifierConfidence::Low,
            },
        }
    }

    fn make_quantifier_result_with_movement(destination: &str) -> QuantifierResult {
        QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["carla".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult {
                movement_type: Some(MovementType::Entering),
                destination: Some(destination.to_string()),
                confidence: QuantifierConfidence::High,
            },
        }
    }

    #[test]
    fn test_execute_freeaction_impl_no_movement() {
        let mut state = make_test_state();
        let world = make_test_world();
        let map = make_test_map();
        let player = make_test_player();
        let all_npcs = make_test_npcs();
        let history = vec![];

        let result = execute_freeaction_impl(
            &mut state,
            "You examine the room.",
            "examine the room",
            &make_quantifier_result_no_movement(),
            &world,
            &map,
            &player,
            &all_npcs,
            &["carla".to_string()],
            &history,
        );

        assert!(result.is_ok());
        // Narration should be logged
        assert_eq!(state.narration_history.len(), 1);
        assert_eq!(state.narration_history[0].log_type, LogType::Narration);
        // NPCs in area should be updated
        assert_eq!(state.npcs_in_area.len(), 1);
        assert_eq!(state.npcs_in_area[0].id, "carla");
    }

    #[test]
    fn test_execute_freeaction_impl_with_movement() {
        let mut state = make_test_state();
        let world = make_test_world();
        let map = make_test_map();
        let player = make_test_player();
        let all_npcs = make_test_npcs();
        let history = vec![];

        // quantifier result with movement to a new room
        let result = execute_freeaction_impl(
            &mut state,
            "You walk to the tavern.",
            "walk to the tavern",
            &make_quantifier_result_with_movement("nonexistent_room"),
            &world,
            &map,
            &player,
            &all_npcs,
            &["carla".to_string()],
            &history,
        );

        assert!(result.is_ok());
        // Narration logged
        assert!(!state.narration_history.is_empty());
        // Room changed to a dynamic room (since destination doesn't exist)
        assert!(state.current_room_id.starts_with("dynamic_"));
        assert!(state.dynamic_rooms.contains_key(&state.current_room_id));
    }

    #[test]
    fn test_execute_freeaction_impl_updates_npcs_in_area() {
        let mut state = make_test_state();
        let world = make_test_world();
        let map = make_test_map();
        let player = make_test_player();
        let all_npcs = make_test_npcs();

        // Start with empty npcs_in_area
        assert!(state.npcs_in_area.is_empty());

        let result = execute_freeaction_impl(
            &mut state,
            "You look around.",
            "look around",
            &make_quantifier_result_no_movement(),
            &world,
            &map,
            &player,
            &all_npcs,
            &["carla".to_string()],
            &vec![],
        );

        assert!(result.is_ok());
        // npcs_in_area should now contain carla
        assert_eq!(state.npcs_in_area.len(), 1);
        assert_eq!(state.npcs_in_area[0].id, "carla");
    }

    #[test]
    fn test_execute_freeaction_impl_triggers_evaluated() {
        // Create NPC with a trigger
        let npc = NpcCard {
            id: "carla".to_string(),
            sheet: CharacterSheet {
                name: "Carla".to_string(),
                description: "A friendly NPC".to_string(),
                personality: "Friendly".to_string(),
                scenario: "Test scenario".to_string(),
                example_dialogue: "Hello!".to_string(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![crate::model::trigger::Trigger {
                condition: crate::model::trigger::TriggerCondition::TimesMet(
                    crate::model::trigger::ComparisonOperator::Eq,
                    0,
                ),
                action: crate::model::trigger::TriggerAction {
                    narration_prompt: "Carla greets you warmly!".to_string(),
                },
                repeat: false,
            }],
        };

        let world = make_test_world();
        let map = make_test_map();
        let player = make_test_player();

        let mut npcs = std::collections::HashMap::new();
        npcs.insert("carla".to_string(), npc);

        let mut state = GameState {
            world: world.clone(),
            map: map.clone(),
            player: player.clone(),
            npcs,
            current_room_id: "room1".to_string(),
            narration_history: vec![],
            next_log_id: 1,
            npcs_in_area: vec![],
            dynamic_rooms: std::collections::HashMap::new(),
            character_state: Default::default(),
            generation_state: Default::default(),
        };

        // NPC has TimesMet Eq 0 trigger - should fire because times_met starts at 0
        // Note: evaluate_and_narrate_triggers calls LLM internally, so this test
        // will use whatever backend is configured (mock in test env)
        let result = execute_freeaction_impl(
            &mut state,
            "You enter the room.",
            "enter",
            &make_quantifier_result_no_movement(),
            &world,
            &map,
            &player,
            &[], // all_npcs - empty won't match triggers
            &["carla".to_string()],
            &vec![],
        );

        // Result should be ok (even if trigger LLM call fails, fn handles gracefully)
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_freeaction_impl_npc_events_entered() {
        let mut state = make_test_state();
        // NPC already in area (simulating re-encounter after leaving)
        state.npcs_in_area = vec![]; // Empty - NPC is not currently in area
        let world = make_test_world();
        let map = make_test_map();
        let player = make_test_player();
        let all_npcs = make_test_npcs();

        let result = execute_freeaction_impl(
            &mut state,
            "You see Carla.",
            "look around",
            &make_quantifier_result_no_movement(),
            &world,
            &map,
            &player,
            &all_npcs,
            &["carla".to_string()],
            &vec![],
        );

        assert!(result.is_ok());
        // NPC enters - times_met should increment
        let times_met = state
            .character_state
            .npcs
            .get("carla")
            .map(|s| s.times_met)
            .unwrap_or(0);
        assert_eq!(times_met, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::trigger_eval::{get_times_met, is_currently_meeting};
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

        assert!(is_currently_meeting(&state.character_state, "carla"));
    }

    #[test]
    fn test_apply_npc_events_left() {
        let mut state = make_test_state();
        set_currently_meeting(&mut state.character_state, "carla", true);

        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Left,
        }];

        apply_npc_events(&mut state, &events);

        assert!(!is_currently_meeting(&state.character_state, "carla"));
    }

    #[test]
    fn test_apply_npc_events_increments_times_met() {
        let mut state = make_test_state();
        let initial_times = get_times_met(&state.character_state, "carla");

        let events = vec![NpcEvent {
            npc_id: "carla".to_string(),
            event_type: crate::narrative::quantifier::NpcEventType::Entered,
        }];

        apply_npc_events(&mut state, &events);

        assert_eq!(
            get_times_met(&state.character_state, "carla"),
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
        let initial_times = get_times_met(&state.character_state, "carla");

        handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

        // times_met should not increment when room doesn't change
        assert_eq!(
            get_times_met(&state.character_state, "carla"),
            initial_times
        );
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

        handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

        assert!(!state.narration_history.is_empty());
        let last_entry = state.narration_history.last().unwrap();
        assert_eq!(last_entry.log_type, LogType::Narration);
        assert_eq!(last_entry.sender, Some("Test Room".to_string()));
    }

    #[test]
    fn test_handle_movement_sets_currently_meeting() {
        let mut state = make_test_state();

        handle_movement(&mut state, Some("new_room"), &["carla".to_string()]);

        // Should set currently_meeting for NPCs in new room
        assert!(is_currently_meeting(&state.character_state, "carla"));
    }
}
