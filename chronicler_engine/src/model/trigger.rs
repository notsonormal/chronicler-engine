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
    pub name: String,
    pub narration_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    pub condition: TriggerCondition,
    pub action: TriggerAction,
    pub repeat: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<usize, bool>,
    pub currently_meeting: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterState {
    pub npcs: HashMap<String, NpcEncounterState>,
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
        let json = r#"{"name": "Old Friend", "narration_prompt": "You meet an old friend."}"#;
        let action: TriggerAction = serde_json::from_str(json).unwrap();
        assert_eq!(action.name, "Old Friend");
        assert_eq!(action.narration_prompt, "You meet an old friend.");
    }

    #[test]
    fn test_trigger_serde_without_room_id() {
        let json = r#"{
            "condition": {"TimesMet": ["Gte", 2]},
            "action": {"name": "Guard Recognition", "narration_prompt": "The guard recognizes you."},
            "repeat": false
        }"#;
        let trigger: Trigger = serde_json::from_str(json).unwrap();
        assert_eq!(
            trigger.condition,
            TriggerCondition::TimesMet(ComparisonOperator::Gte, 2)
        );
        assert_eq!(trigger.action.narration_prompt, "The guard recognizes you.");
        assert!(!trigger.repeat);
        assert_eq!(trigger.room_id, None);
    }

    #[test]
    fn test_trigger_serde_with_room_id() {
        let json = r#"{
            "condition": {"TimesMet": ["Eq", 0]},
            "action": {"name": "Introduction", "narration_prompt": "They appear."},
            "repeat": false,
            "room_id": "entrance_hall"
        }"#;
        let trigger: Trigger = serde_json::from_str(json).unwrap();
        assert_eq!(
            trigger.condition,
            TriggerCondition::TimesMet(ComparisonOperator::Eq, 0)
        );
        assert_eq!(trigger.room_id, Some("entrance_hall".to_string()));
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
        let mut state = NpcEncounterState {
            times_met: 5,
            ..Default::default()
        };
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
                currently_meeting: false,
            },
        );

        assert!(state.npcs.contains_key("carla"));
        assert_eq!(state.npcs.get("carla").unwrap().times_met, 3);
    }
}
