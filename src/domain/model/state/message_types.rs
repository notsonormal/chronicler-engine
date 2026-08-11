//! [DOC: docs/diataxis/reference/game_flow.md]
//! Message type and entry definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Narration,
    Dialogue,
    System,
    Input,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageEntry {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub location_header: Option<String>,
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
            sender: None,
            text: String::new(),
            message_type: MessageType::Narration,
            timestamp: Utc::now(),
            location_header: None,
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
            sender: msg.sender.clone(),
            text: msg.text().to_string(),
            message_type: msg.message_type.clone(),
            timestamp: msg.timestamp,
            location_header: msg.location_header().map(|s| s.to_string()),
            event_header: msg.event_header().map(|s| s.to_string()),
            swipe_count: msg.swipe_count(),
            active_swipe_index: msg.active_swipe_index,
        }
    }
}
