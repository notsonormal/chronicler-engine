use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::state::LogType;

/// A single message in the narrative history.
///
/// Unlike the old [`Turn`](super::turn::Turn) model, each AI output is its
/// own independent message with its own set of swipes.  This allows retrying
/// or deleting individual responses (narration, event, dialogue) without
/// affecting earlier messages in the same turn.
///
/// Messages are stored in chronological order.  Only the **last** message may
/// be retried or deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier (monotonically increasing).
    pub id: u64,
    /// The turn this message belongs to. All messages created between
    /// user inputs share the same turn_id.
    pub turn_id: String,
    /// Who sent this message (``None`` for the narrator).
    pub sender: Option<String>,
    /// The text content.
    pub text: String,
    /// What kind of message this is.
    pub log_type: LogType,
    /// When the message was created.
    pub timestamp: DateTime<Utc>,
    /// Optional location-change header (shown when the player moves).
    pub location_header: Option<String>,
    /// Optional event header (shown when a trigger fires).
    pub event_header: Option<String>,
    /// All swipes (variants) for this message.
    pub swipes: Vec<MessageSwipe>,
    /// Index of the currently visible swipe in [`swipes`].
    pub active_swipe_index: u32,
}

/// A single swipe (variant) for a [`Message`].
///
/// When the user hits "Retry", a new swipe is created for the **last**
/// message, leaving all earlier messages untouched.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSwipe {
    /// Variant number (0-based, monotonically increasing within a message).
    pub index: u32,
    /// The text content of this swipe.
    pub text: String,
}

impl Message {
    /// Create a new message with a single default swipe.
    pub fn new(
        id: u64,
        turn_id: impl Into<String>,
        sender: Option<String>,
        text: impl Into<String>,
        log_type: LogType,
        location_header: Option<String>,
        event_header: Option<String>,
    ) -> Self {
        let text = text.into();
        Self {
            id,
            turn_id: turn_id.into(),
            sender,
            text: text.clone(),
            log_type,
            timestamp: Utc::now(),
            location_header,
            event_header,
            swipes: vec![MessageSwipe { index: 0, text }],
            active_swipe_index: 0,
        }
    }

    /// The currently active swipe, if any.
    pub fn active_swipe(&self) -> Option<&MessageSwipe> {
        self.swipes.get(self.active_swipe_index as usize)
    }

    /// Mutable reference to the currently active swipe.
    pub fn active_swipe_mut(&mut self) -> Option<&mut MessageSwipe> {
        self.swipes.get_mut(self.active_swipe_index as usize)
    }

    /// Text of the active swipe (or empty string if none).
    pub fn active_text(&self) -> &str {
        self.active_swipe().map(|s| s.text.as_str()).unwrap_or("")
    }

    /// Create a new swipe with the given text and switch to it.
    pub fn create_swipe(&mut self, text: impl Into<String>) {
        let index = self.swipes.len() as u32;
        self.swipes.push(MessageSwipe {
            index,
            text: text.into(),
        });
        self.active_swipe_index = index;
    }

    /// Create a new swipe that copies the active swipe's text.
    pub fn create_swipe_copying_active(&mut self) {
        let text = self.active_text().to_string();
        self.create_swipe(text);
    }

    /// Switch to a different swipe by index.
    pub fn switch_swipe(&mut self, index: u32) -> bool {
        if (index as usize) < self.swipes.len() {
            self.active_swipe_index = index;
            self.text = self.active_text().to_string();
            true
        } else {
            false
        }
    }
}
