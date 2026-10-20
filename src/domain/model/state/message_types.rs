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
    /// The player dictating their own Persona's Message.
    ImpersonateInput,
    /// A Narration the narrator wrote.
    Narration,
    /// A Narration a Trigger started, with the record it came from.
    EventNarration(EventRecord),
}

/// The story reads take no Swipe state: every field here is a Message's own text
/// or a header it shows.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryRow {
    pub id: u64,
    pub kind: MessageKind,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub room_report: Option<String>,
    pub event_header: Option<String>,
    pub swipe_count: usize,
    pub active_swipe_index: usize,
}

impl From<&StoryRow> for MessageEntry {
    fn from(row: &StoryRow) -> Self {
        Self {
            id: row.id,
            text: row.text.clone(),
            message_type: row.message_type.clone(),
            timestamp: row.timestamp,
            room_report: row.room_report.clone(),
            event_header: row.event_header.clone(),
            swipe_count: row.swipe_count,
            active_swipe_index: row.active_swipe_index,
        }
    }
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
