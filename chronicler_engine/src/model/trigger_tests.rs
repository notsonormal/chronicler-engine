use std::collections::HashMap;

use crate::model::trigger::{
    ComparisonOperator, NpcEncounterLog, NpcEncounterState, Trigger, TriggerNarration,
    TriggerRequirement,
};

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
fn test_trigger_requirement_serde() {
    let json = r#"{"operator": "Eq", "threshold": 3}"#;
    let cond: TriggerRequirement = serde_json::from_str(json).unwrap();
    assert_eq!(
        cond,
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 3
        }
    );
}

#[test]
fn test_trigger_narration_serde() {
    let json = r#"{"name": "Old Friend", "narration_prompt": "You meet an old friend."}"#;
    let narration: TriggerNarration = serde_json::from_str(json).unwrap();
    assert_eq!(narration.name, "Old Friend");
    assert_eq!(narration.narration_prompt, "You meet an old friend.");
}

#[test]
fn test_trigger_serde_without_room_id() {
    let json = r#"{
        "requirement": {"operator": "Gte", "threshold": 2},
        "narration": {"name": "Guard Recognition", "narration_prompt": "The guard recognizes you."},
        "repeat": false
    }"#;
    let trigger: Trigger = serde_json::from_str(json).unwrap();
    assert_eq!(
        trigger.requirement,
        TriggerRequirement {
            operator: ComparisonOperator::Gte,
            threshold: 2
        }
    );
    assert_eq!(
        trigger.narration.narration_prompt,
        "The guard recognizes you."
    );
    assert!(!trigger.repeat);
    assert_eq!(trigger.room_id, None);
}

#[test]
fn test_trigger_serde_with_room_id() {
    let json = r#"{
        "requirement": {"operator": "Eq", "threshold": 0},
        "narration": {"name": "Introduction", "narration_prompt": "They appear."},
        "repeat": false,
        "room_id": "entrance_hall"
    }"#;
    let trigger: Trigger = serde_json::from_str(json).unwrap();
    assert_eq!(
        trigger.requirement,
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0
        }
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
fn test_npc_encounter_log_default() {
    let state = NpcEncounterLog::default();
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
fn test_npc_encounter_log_npc_tracking() {
    let mut state = NpcEncounterLog::default();
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
