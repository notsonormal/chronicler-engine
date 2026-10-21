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

/// The one story entry. It takes no Swipe state: every field here is a
/// Message's own text or a header it shows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StoryRow {
    pub id: u64,
    pub kind: MessageKind,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub event_header: Option<String>,
    pub swipe_count: usize,
    pub active_swipe_index: usize,
}
