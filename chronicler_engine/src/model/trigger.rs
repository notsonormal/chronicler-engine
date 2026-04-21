use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Eq,
    Lt,
    Gte,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerCondition {
    TimesMet(ComparisonOperator, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerAction {
    pub narration_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    pub condition: TriggerCondition,
    pub action: TriggerAction,
    pub repeat: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<usize, bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterState {
    pub npcs: HashMap<String, NpcEncounterState>,
}

impl CharacterState {
    // [DOC: docs/system/character_state.md]
    pub fn get_times_met(&self, npc_id: &str) -> u32 {
        self.npcs.get(npc_id).map(|s| s.times_met).unwrap_or(0)
    }

    pub fn increment_times_met(&mut self, npc_id: &str) {
        let entry = self.npcs.entry(npc_id.to_string()).or_default();
        entry.times_met += 1;
    }

    pub fn is_trigger_fired(&self, npc_id: &str, trigger_index: usize) -> bool {
        self.npcs
            .get(npc_id)
            .and_then(|s| s.trigger_fired.get(&trigger_index))
            .copied()
            .unwrap_or(false)
    }

    pub fn mark_trigger_fired(&mut self, npc_id: &str, trigger_index: usize) {
        let entry = self.npcs.entry(npc_id.to_string()).or_default();
        entry.trigger_fired.insert(trigger_index, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_operator_serde() {
        let json = r#""Eq""#;
        let op: ComparisonOperator = serde_json::from_str(json).unwrap();
        assert_eq!(op, ComparisonOperator::Eq);

        let json = r#""Lt""#;
        let op: ComparisonOperator = serde_json::from_str(json).unwrap();
        assert_eq!(op, ComparisonOperator::Lt);

        let json = r#""Gte""#;
        let op: ComparisonOperator = serde_json::from_str(json).unwrap();
        assert_eq!(op, ComparisonOperator::Gte);
    }

    #[test]
    fn test_trigger_condition_serde() {
        let json = r#"{"TimesMet": ["Eq", 3]}"#;
        let cond: TriggerCondition = serde_json::from_str(json).unwrap();
        assert_eq!(cond, TriggerCondition::TimesMet(ComparisonOperator::Eq, 3));
    }

    #[test]
    fn test_trigger_action_serde() {
        let json = r#"{"narration_prompt": "You meet an old friend."}"#;
        let action: TriggerAction = serde_json::from_str(json).unwrap();
        assert_eq!(action.narration_prompt, "You meet an old friend.");
    }

    #[test]
    fn test_trigger_serde() {
        let json = r#"{
            "condition": {"TimesMet": ["Gte", 2]},
            "action": {"narration_prompt": "The guard recognizes you."},
            "repeat": false
        }"#;
        let trigger: Trigger = serde_json::from_str(json).unwrap();
        assert_eq!(
            trigger.condition,
            TriggerCondition::TimesMet(ComparisonOperator::Gte, 2)
        );
        assert_eq!(trigger.action.narration_prompt, "The guard recognizes you.");
        assert!(!trigger.repeat);
    }

    #[test]
    fn test_npc_encounter_state_default() {
        let state = NpcEncounterState::default();
        assert_eq!(state.times_met, 0);
        assert!(state.trigger_fired.is_empty());
    }

    #[test]
    fn test_character_state_default() {
        let state = CharacterState::default();
        assert!(state.npcs.is_empty());
    }

    #[test]
    fn test_npc_encounter_state_update() {
        let mut state = NpcEncounterState::default();
        state.times_met = 5;
        state.trigger_fired.insert(0, true);
        state.trigger_fired.insert(1, false);

        assert_eq!(state.times_met, 5);
        assert!(state.trigger_fired.get(&0).copied().unwrap_or(false));
        assert!(!state.trigger_fired.get(&1).copied().unwrap_or(true));
    }

    #[test]
    fn test_character_state_npc_tracking() {
        let mut state = CharacterState::default();
        state.npcs.insert(
            "carla".to_string(),
            NpcEncounterState {
                times_met: 3,
                trigger_fired: HashMap::new(),
            },
        );

        assert!(state.npcs.contains_key("carla"));
        assert_eq!(state.npcs.get("carla").unwrap().times_met, 3);
    }
}
