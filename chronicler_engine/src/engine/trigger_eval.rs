use crate::model::character::NpcCard;
use crate::model::state::GameState;
use crate::model::trigger::{CharacterState, ComparisonOperator, Trigger, TriggerCondition};

/// [DOC: docs/system/triggers.md]
pub fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger)> {
    let mut results = Vec::new();

    for npc in state.npcs.values() {
        for (index, trigger) in npc.triggers.iter().enumerate() {
            if check_condition(&state.character_state, &npc.id, &trigger.condition) {
                if !trigger.repeat && is_trigger_fired(&state.character_state, &npc.id, index) {
                    continue;
                }
                results.push((npc.clone(), trigger.clone()));
            }
        }
    }

    results
}

pub fn check_condition(
    character_state: &crate::model::trigger::CharacterState,
    npc_id: &str,
    condition: &TriggerCondition,
) -> bool {
    match condition {
        TriggerCondition::TimesMet(op, threshold) => {
            let times_met = get_times_met(character_state, npc_id);
            match op {
                ComparisonOperator::Eq => times_met == *threshold,
                ComparisonOperator::Lt => times_met < *threshold,
                ComparisonOperator::Gte => times_met >= *threshold,
            }
        }
    }
}

pub fn is_currently_meeting(character_state: &CharacterState, npc_id: &str) -> bool {
    character_state
        .npcs
        .get(npc_id)
        .map(|s| s.currently_meeting)
        .unwrap_or(false)
}

pub fn increment_times_met(character_state: &mut CharacterState, npc_id: &str) {
    let entry = character_state.npcs.entry(npc_id.to_string()).or_default();
    entry.times_met += 1;
}

pub fn mark_trigger_fired(
    character_state: &mut CharacterState,
    npc_id: &str,
    trigger_index: usize,
) {
    let entry = character_state.npcs.entry(npc_id.to_string()).or_default();
    entry.trigger_fired.insert(trigger_index, true);
}

pub fn set_currently_meeting(character_state: &mut CharacterState, npc_id: &str, meeting: bool) {
    let entry = character_state.npcs.entry(npc_id.to_string()).or_default();
    entry.currently_meeting = meeting;
}

pub fn get_times_met(character_state: &CharacterState, npc_id: &str) -> u32 {
    character_state
        .npcs
        .get(npc_id)
        .map(|s| s.times_met)
        .unwrap_or(0)
}

pub fn is_trigger_fired(
    character_state: &CharacterState,
    npc_id: &str,
    trigger_index: usize,
) -> bool {
    character_state
        .npcs
        .get(npc_id)
        .and_then(|s| s.trigger_fired.get(&trigger_index))
        .copied()
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::{CharacterSheet, NpcCard};
    use crate::model::map::{MapDef, Overworld};
    use crate::model::state::GameState;
    use crate::model::trigger::{CharacterState, Trigger, TriggerAction, TriggerCondition};
    use crate::model::world::WorldCard;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_npc(id: &str, triggers: Vec<Trigger>) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: CharacterSheet {
                name: id.to_string(),
                description: "Test NPC".to_string(),
                personality: "Neutral".to_string(),
                scenario: "Testing".to_string(),
                example_dialogue: String::new(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers,
        }
    }

    fn make_trigger(condition: TriggerCondition, repeat: bool) -> Trigger {
        Trigger {
            condition,
            action: TriggerAction {
                name: "Test Event".to_string(),
                narration_prompt: "Test trigger".to_string(),
            },
            repeat,
        }
    }

    fn make_state(
        npcs_in_area: Vec<NpcCard>,
        all_npcs: &[NpcCard],
        character_state: CharacterState,
    ) -> GameState {
        let world = Arc::new(WorldCard {
            name: "Test".into(),
            description: "Test world".into(),
            global_rules: vec![],
            ..Default::default()
        });
        let map = Arc::new(MapDef {
            overworld: Overworld {
                id: "ow".into(),
                name: "ow".into(),
                regions: vec![],
            },
        });
        let player = Arc::new(crate::model::character::PlayerCard {
            sheet: CharacterSheet {
                name: "Player".into(),
                description: "Test player".into(),
                personality: "Brave".to_string(),
                scenario: "Testing".to_string(),
                example_dialogue: String::new(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        });
        let mut npcs = HashMap::new();
        for npc in all_npcs {
            npcs.insert(npc.id.clone(), npc.clone());
        }
        GameState {
            world,
            map,
            player,
            npcs,
            current_room_id: "room_1".into(),
            narration_history: vec![],
            next_log_id: 1,
            npcs_in_area,
            generation_state: Default::default(),
            dynamic_rooms: HashMap::new(),
            character_state,
        }
    }

    #[test]
    fn test_evaluate_triggers_empty_room() {
        let state = make_state(vec![], &[], CharacterState::default());
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_first_encounter() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        let state = make_state(vec![npc.clone()], &[npc.clone()], CharacterState::default());
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "gabriella");
    }

    #[test]
    fn test_evaluate_triggers_no_match_when_times_met_greater() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        let mut character_state = CharacterState::default();
        increment_times_met(&mut character_state, "gabriella");
        let state = make_state(vec![npc.clone()], &[npc.clone()], character_state);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_skips_fired_non_repeatable() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger.clone()]);
        let mut character_state = CharacterState::default();
        mark_trigger_fired(&mut character_state, "gabriella", 0);
        let state = make_state(vec![npc.clone()], &[npc.clone()], character_state);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_fires_for_npc_not_in_area() {
        // Test that triggers fire for NPCs even when they're NOT in npcs_in_area.
        /// [DOC: docs/architecture/system.md]
        // NOTE: Triggers must check all NPCs in state.npcs, not just npcs_in_area.
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        // npcs_in_area is empty, but Gabriella IS in state.npcs
        let state = make_state(vec![], &[npc.clone()], CharacterState::default());

        // Verify Gabriella IS in state.npcs but NOT in npcs_in_area
        assert!(
            state.npcs.contains_key("gabriella"),
            "Gabriella should be in state.npcs"
        );
        assert!(
            state.npcs_in_area.is_empty(),
            "npcs_in_area should be empty"
        );

        // Trigger should still fire because we check ALL npcs, not just npcs_in_area
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1, "Trigger should fire for NPC not in area");
        assert_eq!(results[0].0.id, "gabriella");
    }

    #[test]
    fn test_evaluate_triggers_repeatable_fires_again() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Lt, 3), true);
        let npc = make_npc("ranger", vec![trigger]);
        let mut character_state = CharacterState::default();
        increment_times_met(&mut character_state, "ranger");
        let state = make_state(vec![npc.clone()], &[npc.clone()], character_state);
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_currently_meeting_tracks_encounters() {
        let mut character_state = CharacterState::default();

        // Initially not meeting
        assert!(!is_currently_meeting(&character_state, "carla"));

        // Start meeting
        set_currently_meeting(&mut character_state, "carla", true);
        assert!(is_currently_meeting(&character_state, "carla"));

        // End meeting (player leaves room)
        set_currently_meeting(&mut character_state, "carla", false);
        assert!(!is_currently_meeting(&character_state, "carla"));
    }

    #[test]
    fn test_increment_times_met_always_increments() {
        // increment_times_met always increments - the guard is in handle_movement
        let mut character_state = CharacterState::default();

        // Always increments when called
        increment_times_met(&mut character_state, "gabriella");
        assert_eq!(get_times_met(&character_state, "gabriella"), 1);

        // Can be called multiple times
        increment_times_met(&mut character_state, "gabriella");
        assert_eq!(get_times_met(&character_state, "gabriella"), 2);
    }

    #[test]
    fn test_character_state_initializes_with_starting_room_npcs() {
        // Test that NPCs in the starting room have times_met=1 and currently_meeting=true
        use crate::model::character::NpcCard;
        use crate::model::map::{MapDef, Overworld, Region, Room};
        use crate::model::world::WorldCard;
        use std::sync::Arc;

        let world = Arc::new(WorldCard {
            name: "Test".into(),
            description: "Test".into(),
            ..Default::default()
        });

        let room = Room {
            id: "start".into(),
            name: "Start".into(),
            description: "A room".into(),
            exits: HashMap::new(),
            items: vec![],
            npcs: vec!["carla".into()],
            image_path: None,
            navigation_description: None,
        };
        let region = Region {
            id: "reg".into(),
            name: "Region".into(),
            rooms: vec![room],
        };
        let map = Arc::new(MapDef {
            overworld: Overworld {
                id: "ow".into(),
                name: "Overworld".into(),
                regions: vec![region],
            },
        });

        let npc = NpcCard {
            id: "carla".into(),
            sheet: crate::model::character::CharacterSheet {
                name: "Carla".into(),
                description: "Bodyguard".into(),
                personality: "Protective".into(),
                scenario: "Guarding".into(),
                example_dialogue: "Stay safe.".into(),
                profile_image: None,
                headshot_image: None,
            },
            triggers: vec![],
            inventory: vec![],
        };
        let npcs = vec![npc];

        let state = crate::model::state::GameState::new(
            world,
            map,
            Arc::new(crate::model::character::PlayerCard {
                sheet: crate::model::character::CharacterSheet {
                    name: "Player".into(),
                    description: "".into(),
                    personality: "Brave".into(),
                    scenario: "Test".into(),
                    example_dialogue: "".into(),
                    profile_image: None,
                    headshot_image: None,
                },
                inventory: vec![],
            }),
            npcs,
            "start".into(),
        );

        // Carla in starting room should have times_met=1
        assert_eq!(get_times_met(&state.character_state, "carla"), 1);
        assert!(is_currently_meeting(&state.character_state, "carla"));
    }

    #[test]
    fn test_increment_times_met_and_mark_fired() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        let mut state = make_state(vec![npc.clone()], &[npc.clone()], CharacterState::default());
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1);
        increment_times_met(&mut state.character_state, "gabriella");
        mark_trigger_fired(&mut state.character_state, "gabriella", 0);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_check_condition_missing_npc_defaults_to_zero() {
        let character_state = CharacterState::default();
        let condition = TriggerCondition::TimesMet(ComparisonOperator::Eq, 0);
        assert!(check_condition(&character_state, "unknown_npc", &condition));
    }
}
