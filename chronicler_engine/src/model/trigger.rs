//! [DOC: docs/system/triggers.md]
//! Trigger conditions and event types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Eq,
    Lt,
    Gte,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerRequirement {
    TimesMet(ComparisonOperator, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerNarration {
    pub name: String,
    pub narration_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    pub requirement: TriggerRequirement,
    pub narration: TriggerNarration,
    pub repeat: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
}

/// Per-NPC encounter tracking state.
///
/// Tracks encounter cycles for a single NPC: entering → exiting → re-entering.
/// - `times_met`: Increments on first encounter (Entered from not currently_meeting)
/// - `currently_meeting`: Set true on Entered, false on Left
/// - `trigger_fired`: Indices of non-repeatable triggers that have fired
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
