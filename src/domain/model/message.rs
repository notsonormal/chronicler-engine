//! [DOC: docs/diataxis/reference/storage.md]
//! Message types and conversation history (Message, Swipe, stored generation inputs)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::model::state::message_types::MessageType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub text: String,
    pub snapshot_id: Option<u64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    #[serde(default)]
    pub impersonated: bool,
    /// Holds the guide text (plain generation) or the impersonate direction;
    /// `impersonated` is the discriminator — the two never coexist.
    #[serde(default)]
    pub steering_instruction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub active_swipe_index: usize,
    pub swipes: Vec<Swipe>,
    pub is_deleted: bool,
}

impl Message {
    pub fn new(
        text: impl Into<String>,
        message_type: MessageType,
        location_header: Option<String>,
        event_header: Option<String>,
    ) -> Self {
        let text = text.into();
        let swipe = Swipe {
            text,
            snapshot_id: None,
            location_header,
            event_header,
            impersonated: false,
            steering_instruction: None,
        };
        Self {
            id: 0,
            message_type,
            timestamp: Utc::now(),
            active_swipe_index: 0,
            swipes: vec![swipe],
            is_deleted: false,
        }
    }

    pub fn is_unpersisted(&self) -> bool {
        self.id == 0
    }

    pub fn swipe_count(&self) -> usize {
        self.swipes.len()
    }

    fn active_swipe(&self) -> Option<&Swipe> {
        self.swipes.get(self.active_swipe_index)
    }

    fn active_swipe_mut(&mut self) -> Option<&mut Swipe> {
        self.swipes.get_mut(self.active_swipe_index)
    }

    pub fn text(&self) -> &str {
        self.active_swipe().map(|s| s.text.as_str()).unwrap_or("")
    }

    pub fn location_header(&self) -> Option<&str> {
        self.active_swipe()
            .and_then(|s| s.location_header.as_deref())
    }

    pub fn event_header(&self) -> Option<&str> {
        self.active_swipe().and_then(|s| s.event_header.as_deref())
    }

    pub fn snapshot_id(&self) -> Option<u64> {
        self.active_swipe().and_then(|s| s.snapshot_id)
    }

    pub fn impersonated(&self) -> bool {
        self.active_swipe().is_some_and(|s| s.impersonated)
    }

    pub fn steering_instruction(&self) -> Option<&str> {
        self.active_swipe()
            .and_then(|s| s.steering_instruction.as_deref())
    }

    pub fn is_input_or_narration(&self) -> bool {
        self.message_type == MessageType::Narration || self.message_type == MessageType::Input
    }

    pub fn is_guided(&self) -> bool {
        !self.impersonated() && self.steering_instruction().is_some()
    }

    pub fn set_active_swipe(&mut self, index: usize) {
        if index >= self.swipes.len() {
            return;
        }
        self.active_swipe_index = index;
    }

    pub fn update_active_swipe_text(&mut self, new_text: impl Into<String>) {
        let new_text = new_text.into();
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.text = new_text;
        }
    }

    pub fn set_snapshot_id(&mut self, sid: Option<u64>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.snapshot_id = sid;
        }
    }

    pub(crate) fn from_db(
        id: u64,
        message_type: MessageType,
        timestamp: DateTime<Utc>,
        active_swipe_index: usize,
        is_deleted: bool,
    ) -> Self {
        Self {
            id,
            message_type,
            timestamp,
            active_swipe_index,
            swipes: Vec::new(),
            is_deleted,
        }
    }

    /// Returns true when the index was out of bounds and has been reset.
    pub fn ensure_valid_swipe_index(&mut self) -> bool {
        if self.active_swipe_index >= self.swipes.len() {
            self.active_swipe_index = 0;
            true
        } else {
            false
        }
    }
}

impl Message {
    #[cfg(test)]
    pub(crate) fn set_event_header(&mut self, header: Option<String>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.event_header = header;
        }
    }

    pub fn set_stored_inputs(&mut self, impersonated: bool, steering_instruction: Option<String>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.impersonated = impersonated;
            swipe.steering_instruction = steering_instruction;
        }
    }
}
