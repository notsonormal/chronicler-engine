//! [DOC: docs/diataxis/reference/game_flow.md]
//! Message type and entry definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::trigger_context::EventRecord;

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Narration,
    System,
    Input,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageKind {
    /// Free text the player typed.
    TypedInput,
    /// The player dictating their own character's turn.
    ImpersonateInput,
    /// A Narration the narrator wrote.
    Narration,
    /// A Narration a Trigger started, with the record it came from.
    EventNarration(EventRecord),
}

/// One Message of a story read: the entry the narrator's window, the analysis
/// window, the Options window and the story log read. It carries no Swipe state.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryRow {
    pub id: u64,
    /// The kind of the Message. `None` for a row today's code wrote.
    pub kind: Option<MessageKind>,
    pub message_type: MessageType,
    /// The text of the active Swipe.
    pub text: String,
    pub room_report: Option<String>,
    pub event_header: Option<String>,
    pub swipe_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageEntry {
    pub id: u64,
    pub text: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub room_report: Option<String>,
    #[serde(default)]
    pub event_header: Option<String>,
    #[serde(default)]
    pub swipe_count: usize,
    #[serde(default)]
    pub active_swipe_index: usize,
}

impl Default for MessageEntry {
    fn default() -> Self {
        Self {
            id: 0,
            text: String::new(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            room_report: None,
            event_header: None,
            swipe_count: 1,
            active_swipe_index: 0,
        }
    }
}

impl From<&crate::domain::model::message::Message> for MessageEntry {
    fn from(msg: &crate::domain::model::message::Message) -> Self {
        Self {
            id: msg.id,
            text: msg.text().to_string(),
            message_type: msg.message_type.clone(),
            timestamp: msg.timestamp,
            room_report: msg.room_report().map(|s| s.to_string()),
            event_header: msg.event_header().map(|s| s.to_string()),
            swipe_count: msg.swipe_count(),
            active_swipe_index: msg.active_swipe_index,
        }
    }
}
