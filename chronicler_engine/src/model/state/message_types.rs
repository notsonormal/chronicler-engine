//! [DOC: docs/system/game_flow.md]
//! Message type and entry definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
