//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Trigger reference of a Swipe state and the event record of an event Narration

use serde::{Deserialize, Serialize};

/// A reference only: the prompt is assembled at call time, not stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerRef {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
}

/// The trigger context from which an event Narration was written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub trigger: TriggerRef,
    /// The name the event header shows. The stored key stays `name`.
    #[serde(rename = "name")]
    pub header_name: String,
    pub direction: String,
}

impl From<&crate::domain::model::utils::game_rules::TriggerMatch> for TriggerRef {
    fn from(found: &crate::domain::model::utils::game_rules::TriggerMatch) -> Self {
        Self {
            npc_id: found.npc_id.clone(),
            trigger_idx: found.trigger_idx,
            trigger_name: found.trigger_name.clone(),
            trigger_repeat: found.trigger_repeat,
        }
    }
}

impl From<&crate::domain::model::utils::game_rules::TriggerMatch> for EventRecord {
    fn from(found: &crate::domain::model::utils::game_rules::TriggerMatch) -> Self {
        Self {
            trigger: TriggerRef::from(found),
            header_name: found.trigger_name.clone(),
            direction: found.trigger_narration_prompt.clone(),
        }
    }
}
