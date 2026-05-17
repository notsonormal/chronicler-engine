use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Checkpoint {
    pub id: String,
    pub snapshot_id: u64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
