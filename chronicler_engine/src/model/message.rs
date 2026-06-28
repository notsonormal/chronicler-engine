//! [DOC: docs/system/agent_system.md]
//! Message types and conversation history

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::state::message_types::MessageType;

/// Swipe variant of a [`Message`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub text: String,
    pub snapshot_id: Option<u64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

/// Message in the narrative history.
///
/// Content lives in `swipes[active_swipe_index]`; use getters/setters for access.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender: Option<String>,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub active_swipe_index: usize,
    pub swipes: Vec<Swipe>,
    pub is_deleted: bool,
}

impl Message {
    /// Create new message with a single initial swipe.
    pub fn new(
        sender: Option<String>,
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
        };
        Self {
            id: 0,
            sender,
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
        sender: Option<String>,
        message_type: MessageType,
        timestamp: DateTime<Utc>,
        active_swipe_index: usize,
        is_deleted: bool,
    ) -> Self {
        Self {
            id,
            sender,
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
}
