use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::state::LogType;

/// A single message in the narrative history.
///
/// Messages are stored in chronological order.  Only the **last** message may
/// be retried or deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub turn_id: String,
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

impl Message {
    pub fn new(
        id: u64,
        turn_id: impl Into<String>,
        sender: Option<String>,
        text: impl Into<String>,
        log_type: LogType,
        location_header: Option<String>,
        event_header: Option<String>,
    ) -> Self {
        Self {
            id,
            turn_id: turn_id.into(),
            sender,
            text: text.into(),
            log_type,
            timestamp: Utc::now(),
            location_header,
            event_header,
        }
    }
}
