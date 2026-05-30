use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::MessageType;

/// An inactive generation of a message, preserving both the text and the
/// snapshot of the world state at the time it was committed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub text: String,
    pub snapshot_id: Option<u64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

/// A single message in the narrative history.
///
/// Messages are stored in chronological order. Only the **last** message may
/// be retried, deleted, or swiped.
///
/// `text`, `location_header`, `event_header`, and `snapshot_id` always hold
/// the **active** swipe's values at runtime. Inactive swipes live in `swipes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    #[serde(default)]
    pub snapshot_id: Option<u64>,
    #[serde(default)]
    pub active_swipe_index: usize,
    #[serde(default)]
    pub swipes: Vec<Swipe>,
    #[serde(default)]
    pub is_deleted: bool,
}

impl Message {
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
}
