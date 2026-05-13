use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Checkpoint {
    pub id: String,
    pub turn_id: String,
    pub swipe_index: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
