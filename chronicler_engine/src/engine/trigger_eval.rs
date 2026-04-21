use crate::model::character::NpcCard;
use crate::model::state::GameState;
use crate::model::trigger::{ComparisonOperator, Trigger, TriggerCondition};

/// [DOC: docs/system/triggers.md]
pub fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger)> {
    let mut results = Vec::new();

    for npc in &state.npcs_in_area {
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

    fn make_state(npcs_in_area: Vec<NpcCard>, character_state: CharacterState) -> GameState {
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
        for npc in &npcs_in_area {
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
        let state = make_state(vec![], CharacterState::default());
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_first_encounter() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        let state = make_state(vec![npc], CharacterState::default());
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
        let state = make_state(vec![npc], character_state);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_skips_fired_non_repeatable() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger.clone()]);
        let mut character_state = CharacterState::default();
        character_state.mark_trigger_fired("gabriella", 0);
        let state = make_state(vec![npc], character_state);
        let results = evaluate_triggers(&state);
        assert!(results.is_empty());
    }

    #[test]
    fn test_evaluate_triggers_repeatable_fires_again() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Lt, 3), true);
        let npc = make_npc("ranger", vec![trigger]);
        let mut character_state = CharacterState::default();
        character_state.increment_times_met("ranger");
        let state = make_state(vec![npc], character_state);
        let results = evaluate_triggers(&state);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_increment_times_met_and_mark_fired() {
        let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
        let npc = make_npc("gabriella", vec![trigger]);
        let mut state = make_state(vec![npc], CharacterState::default());
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
