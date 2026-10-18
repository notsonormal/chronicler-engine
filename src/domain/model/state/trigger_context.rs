//! [DOC: docs/diataxis/reference/game_flow.md]
//! Stored trigger snapshot context

use serde::{Deserialize, Serialize};

/// What a Swipe state and an event record name, with no assembled prompt: the
/// prompt is built at call time.
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
    /// The name the event header shows.
    pub name: String,
    pub direction: String,
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
            name: context.trigger_name.clone(),
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
