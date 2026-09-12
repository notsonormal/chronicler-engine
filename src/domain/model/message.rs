//! [DOC: docs/diataxis/reference/storage.md]
//! Message types and conversation history (Message, Swipe, stored generation inputs)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::model::state::message_types::MessageType;

/// Swipe variant of a [`Message`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub text: String,
    pub snapshot_id: Option<u64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    /// The swipe was generated as the player's persona speaking.
    #[serde(default)]
    pub impersonated: bool,
    /// Player-typed steering instruction for this generation: the guide text
    /// (plain generation) or the impersonate direction. `impersonated` is the
    /// discriminator — a directed impersonation and a guide never coexist.
    #[serde(default)]
    pub steering_instruction: Option<String>,
}

/// Message in the narrative history.
///
/// Content lives in `swipes[active_swipe_index]`; use getters/setters for access.
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
    /// Create new message with a single initial swipe.
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

    /// Whether the active swipe was generated as the player's persona speaking.
    pub fn impersonated(&self) -> bool {
        self.active_swipe().is_some_and(|s| s.impersonated)
    }

    /// The active swipe's stored steering instruction (guide text or
    /// impersonate direction).
    pub fn steering_instruction(&self) -> Option<&str> {
        self.active_swipe()
            .and_then(|s| s.steering_instruction.as_deref())
    }

    /// A guided turn: a plain generation with a steering instruction.
    pub fn is_guided(&self) -> bool {
        !self.impersonated() && self.steering_instruction().is_some()
    }

    /// Set active swipe index (content accessors use this).
    pub fn set_active_swipe(&mut self, index: usize) {
        if index >= self.swipes.len() {
            return;
        }
        self.active_swipe_index = index;
    }

    /// Update text of active swipe.
    pub fn update_active_swipe_text(&mut self, new_text: impl Into<String>) {
        let new_text = new_text.into();
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.text = new_text;
        }
    }

    /// Set `snapshot_id` on active swipe (persistence only).
    pub fn set_snapshot_id(&mut self, sid: Option<u64>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.snapshot_id = sid;
        }
    }

    /// Construct message from database values.
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

    /// Validate `active_swipe_index`, reset to 0 if out of bounds. Returns true if reset.
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
    /// Set `event_header` on active swipe (test-only).
    #[doc(hidden)]
    pub fn set_event_header(&mut self, header: Option<String>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.event_header = header;
        }
    }

    /// Set the active swipe's stored generation inputs.
    pub fn set_stored_inputs(&mut self, impersonated: bool, steering_instruction: Option<String>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.impersonated = impersonated;
            swipe.steering_instruction = steering_instruction;
        }
    }
}
