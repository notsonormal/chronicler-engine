//! [DOC: docs/system/game_flow.md]
//! Narrative state with history and input buffer

use serde::{Deserialize, Serialize};

use crate::domain::model::message::Message;
use crate::domain::model::message_history::MessageHistory;
use super::generation_status::InputBuffer;
use super::message_types::MessageEntry;
use super::trigger_context::StoredTriggerContext;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NarrativeState {
    pub history: MessageHistory,
    pub input_buffer: InputBuffer,
    pub last_trigger: Option<StoredTriggerContext>,
    #[serde(default)]
    pub pending_location: Option<String>,
    #[serde(default)]
    pub pending_event: Option<String>,
    #[serde(default)]
    pub last_backend_name: Option<String>,
    #[serde(default)]
    pub last_model_name: Option<String>,
    // Transient — not persisted (pipeline run only).
    #[serde(skip)]
    pub retry_target: Option<Message>,
}

impl NarrativeState {
    pub fn history(&self) -> Vec<MessageEntry> {
        self.history.to_message_entries()
    }

    pub fn from_snapshot(
        snapshot: &crate::adapters::driven::storage::snapshot_blob::NarrativeSnapshot,
    ) -> Self {
        Self {
            history: MessageHistory::new(),
            input_buffer: snapshot.input_buffer.clone(),
            last_trigger: snapshot.last_trigger.clone(),
            pending_location: snapshot.pending_location.clone(),
            pending_event: snapshot.pending_event.clone(),
            last_backend_name: snapshot.last_backend_name.clone(),
            last_model_name: snapshot.last_model_name.clone(),
            retry_target: None,
        }
    }
}
