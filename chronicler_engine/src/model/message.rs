use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::LogType;

/// A single message in the narrative history.
///
/// Messages are stored in chronological order. Only the **last** message may
/// be retried or deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    #[serde(default)]
    pub snapshot_id: Option<u64>,
}

impl Message {
    pub fn new(
        sender: Option<String>,
        text: impl Into<String>,
        log_type: LogType,
        location_header: Option<String>,
        event_header: Option<String>,
    ) -> Self {
        Self {
            id: 0,
            sender,
            text: text.into(),
            log_type,
            timestamp: Utc::now(),
            location_header,
            event_header,
            snapshot_id: None,
        }
    }
}
