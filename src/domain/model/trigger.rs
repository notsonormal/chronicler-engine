//! [DOC: docs/diataxis/reference/game_flow.md]
//! Trigger conditions and event types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Eq,
    Lt,
    Gte,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerRequirement {
    pub operator: ComparisonOperator,
    pub threshold: u32,
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

/// Per-Character encounter state.
///
/// - `times_met`: Increments on first encounter (Entered from !currently_meeting)
/// - `currently_meeting`: Set on Entered, cleared on Left
/// - `trigger_fired`: Fired non-repeatable trigger indices
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Encounter {
    pub times_met: u32,
    pub trigger_fired: HashMap<usize, bool>,
    pub currently_meeting: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Encounters {
    /// The stored key is `npcs`.
    #[serde(rename = "npcs")]
    pub characters: HashMap<String, Encounter>,
}

impl Encounters {
    pub fn increment_times_met(&mut self, character_id: &str) {
        let entry = self.characters.entry(character_id.to_string()).or_default();
        entry.times_met += 1;
    }

    pub fn mark_trigger_fired(&mut self, character_id: &str, trigger_index: usize) {
        let entry = self.characters.entry(character_id.to_string()).or_default();
        entry.trigger_fired.insert(trigger_index, true);
    }

    pub fn set_currently_meeting(&mut self, character_id: &str, meeting: bool) {
        let entry = self.characters.entry(character_id.to_string()).or_default();
        entry.currently_meeting = meeting;
    }

    pub fn get_times_met(&self, character_id: &str) -> u32 {
        self.characters
            .get(character_id)
            .map(|s| s.times_met)
            .unwrap_or(0)
    }

    pub fn is_trigger_fired(&self, character_id: &str, trigger_index: usize) -> bool {
        self.characters
            .get(character_id)
            .and_then(|s| s.trigger_fired.get(&trigger_index))
            .copied()
            .unwrap_or(false)
    }

    pub fn is_currently_meeting(&self, character_id: &str) -> bool {
        self.characters
            .get(character_id)
            .map(|s| s.currently_meeting)
            .unwrap_or(false)
    }

    pub fn check_condition(&self, character_id: &str, condition: &TriggerRequirement) -> bool {
        let times_met = self.get_times_met(character_id);
        match condition.operator {
            ComparisonOperator::Eq => times_met == condition.threshold,
            ComparisonOperator::Lt => times_met < condition.threshold,
            ComparisonOperator::Gte => times_met >= condition.threshold,
        }
    }
}
