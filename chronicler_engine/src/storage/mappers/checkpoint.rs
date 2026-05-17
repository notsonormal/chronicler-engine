use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::model::checkpoint::Checkpoint;
use crate::storage::models::checkpoint::DbCheckpoint;

impl TryFrom<&DbCheckpoint> for Checkpoint {
    type Error = EngineError;

    fn try_from(db: &DbCheckpoint) -> Result<Self, Self::Error> {
        let created_at = DateTime::parse_from_rfc3339(&db.created_at)
            .map_err(|e| {
                EngineError::Config(format!("Failed to parse checkpoint created_at: {e}"))
            })?
            .with_timezone(&Utc);

        Ok(Checkpoint {
            id: db.id.clone(),
            snapshot_id: db.snapshot_id as u64,
            name: db.name.clone(),
            created_at,
        })
    }
}

impl From<&Checkpoint> for DbCheckpoint {
    fn from(cp: &Checkpoint) -> Self {
        Self {
            id: cp.id.clone(),
            snapshot_id: cp.snapshot_id as i64,
            name: cp.name.clone(),
            created_at: cp.created_at.to_rfc3339(),
        }
    }
}
