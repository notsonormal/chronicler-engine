use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::state::MessageType;

/// A swipe variant of a message. Stored in `Message::swipes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub text: String,
    pub snapshot_id: Option<u64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

/// A message in the narrative history.
///
/// Runtime-mirrored fields (`text`, `location_header`, `event_header`,
/// `snapshot_id`) are kept in sync with the active swipe index. Access
/// these through getters; mutate through `set_active_swipe()` or
/// `update_active_swipe_text()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender: Option<String>,
    text: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    location_header: Option<String>,
    event_header: Option<String>,
    snapshot_id: Option<u64>,
    pub active_swipe_index: usize,
    pub swipes: Vec<Swipe>,
    pub is_deleted: bool,
}

impl Message {
    /// Creates a new Message with a single initial swipe.
    pub fn new(
        sender: Option<String>,
        text: impl Into<String>,
        message_type: MessageType,
        location_header: Option<String>,
        event_header: Option<String>,
    ) -> Self {
        let text = text.into();
        let swipe = Swipe {
            text: text.clone(),
            snapshot_id: None,
            location_header: location_header.clone(),
            event_header: event_header.clone(),
        };
        Self {
            id: 0,
            sender,
            text,
            message_type,
            timestamp: Utc::now(),
            location_header,
            event_header,
            snapshot_id: None,
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

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn location_header(&self) -> Option<&str> {
        self.location_header.as_deref()
    }

    pub fn event_header(&self) -> Option<&str> {
        self.event_header.as_deref()
    }

    pub fn snapshot_id(&self) -> Option<u64> {
        self.snapshot_id
    }

    /// Switches the active swipe and syncs all runtime-mirrored fields.
    pub fn set_active_swipe(&mut self, index: usize) {
        if index >= self.swipes.len() {
            return;
        }
        self.active_swipe_index = index;
        let swipe = &self.swipes[index];
        self.text = swipe.text.clone();
        self.location_header = swipe.location_header.clone();
        self.event_header = swipe.event_header.clone();
        self.snapshot_id = swipe.snapshot_id;
    }

    /// Updates text of the active swipe. Maintains msg<->swipe consistency.
    pub fn update_active_swipe_text(&mut self, new_text: impl Into<String>) {
        let new_text = new_text.into();
        self.text = new_text.clone();
        if let Some(swipe) = self.swipes.get_mut(self.active_swipe_index) {
            swipe.text = new_text;
        }
    }

    /// Sets snapshot_id directly (used during persistence).
    pub fn set_snapshot_id(&mut self, sid: Option<u64>) {
        self.snapshot_id = sid;
    }

    /// Creates a message from database values. Internal use only.
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
            text: String::new(),
            message_type,
            timestamp,
            location_header: None,
            event_header: None,
            snapshot_id: None,
            active_swipe_index,
            swipes: Vec::new(),
            is_deleted,
        }
    }

    /// Validates active_swipe_index against swipes.len(), falls back to 0 if invalid.
    /// Returns true if fallback was applied (index was out of bounds).
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
    /// Sets event_header directly (used by tests to simulate event messages).
    #[doc(hidden)]
    pub fn set_event_header(&mut self, header: Option<String>) {
        self.event_header = header;
    }
}
