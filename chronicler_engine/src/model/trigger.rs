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
pub struct TriggerEffect {
    pub name: String,
    pub narration_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    pub condition: TriggerCondition,
    #[serde(rename = "action")]
    pub effect: TriggerEffect,
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
pub struct NpcEncounterLog {
    pub npcs: HashMap<String, NpcEncounterState>,
}
