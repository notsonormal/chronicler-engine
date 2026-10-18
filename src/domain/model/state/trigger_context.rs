//! [DOC: docs/diataxis/reference/game_flow.md]
//! Stored trigger snapshot context

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

impl From<&StoredTriggerContext> for TriggerRef {
    fn from(context: &StoredTriggerContext) -> Self {
        Self {
            npc_id: context.npc_id.clone(),
            trigger_idx: context.trigger_idx,
            trigger_name: context.trigger_name.clone(),
            trigger_repeat: context.trigger_repeat,
        }
    }
}

impl From<&StoredTriggerContext> for EventRecord {
    fn from(context: &StoredTriggerContext) -> Self {
        Self {
            trigger: TriggerRef::from(context),
            header_name: context.trigger_name.clone(),
            direction: context.trigger_narration_prompt.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredTriggerContext {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub trigger_narration_prompt: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: Option<u32>,
}
