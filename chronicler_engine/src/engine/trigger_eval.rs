use crate::model::character::NpcCard;
use crate::model::state::GameState;
use crate::model::trigger::{ComparisonOperator, Trigger, TriggerCondition};

/// [DOC: docs/system/triggers.md]
pub fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger)> {
    let mut results = Vec::new();

    for npc in state.npcs.values() {
        for (index, trigger) in npc.triggers.iter().enumerate() {
            if check_condition(&state.character_state, &npc.id, &trigger.condition) {
                if !trigger.repeat && state.character_state.is_trigger_fired(&npc.id, index) {
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
            let times_met = character_state.get_times_met(npc_id);
            match op {
                ComparisonOperator::Eq => times_met == *threshold,
                ComparisonOperator::Lt => times_met < *threshold,
                ComparisonOperator::Gte => times_met >= *threshold,
            }
        }
    }
}

pub fn increment_times_met(state: &mut GameState, npc_id: &str) {
    // [DOC: docs/system/triggers.md]
    state.character_state.increment_times_met(npc_id);
}

pub fn mark_trigger_fired(state: &mut GameState, npc_id: &str, trigger_index: usize) {
    state
        .character_state
        .mark_trigger_fired(npc_id, trigger_index);
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
        character_state.increment_times_met("gabriella");
        let state = make_state(vec![npc.clone()], &[npc.clone()], character_state);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_skips_fired_non_repeatable() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger.clone()]);
        let mut character_state = CharacterState::default();
        character_state.mark_trigger_fired("gabriella", 0);
        let state = make_state(vec![npc.clone()], &[npc.clone()], character_state);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_fires_for_npc_not_in_area() {
        // Test that triggers fire for NPCs even when they're NOT in npcs_in_area.
        // This catches the bug where only NPCs in npcs_in_area were checked.
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
        character_state.increment_times_met("ranger");
        let state = make_state(vec![npc.clone()], &[npc.clone()], character_state);
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_currently_meeting_tracks_encounters() {
        let mut character_state = CharacterState::default();

        // Initially not meeting
        assert!(!character_state.is_currently_meeting("carla"));

        // Start meeting
        character_state.set_currently_meeting("carla", true);
        assert!(character_state.is_currently_meeting("carla"));

        // End meeting (player leaves room)
        character_state.set_currently_meeting("carla", false);
        assert!(!character_state.is_currently_meeting("carla"));
    }

    #[test]
    fn test_increment_times_met_always_increments() {
        // increment_times_met always increments - the guard is in handle_movement
        let mut character_state = CharacterState::default();

        // Always increments when called
        character_state.increment_times_met("gabriella");
        assert_eq!(character_state.get_times_met("gabriella"), 1);

        // Can be called multiple times
        character_state.increment_times_met("gabriella");
        assert_eq!(character_state.get_times_met("gabriella"), 2);
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
        assert_eq!(state.character_state.get_times_met("carla"), 1);
        assert!(state.character_state.is_currently_meeting("carla"));
    }

    #[test]
    fn test_increment_times_met_and_mark_fired() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        let mut state = make_state(vec![npc.clone()], &[npc.clone()], CharacterState::default());
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1);
        increment_times_met(&mut state, "gabriella");
        mark_trigger_fired(&mut state, "gabriella", 0);
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
