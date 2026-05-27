use std::collections::HashMap;

use crate::error::EngineError;
use crate::model::message::Swipe;
use crate::storage::db::DbPool;

pub trait MessageSwipeStorage: Send + Sync {
    fn insert_swipe(&self, message_id: u64, swipe: &Swipe, index: usize)
    -> Result<(), EngineError>;

    fn update_swipe_text(
        &self,
        message_id: u64,
        swipe_index: usize,
        text: &str,
    ) -> Result<(), EngineError>;

    fn shift_swipe_indices(&self, message_id: u64, offset: usize) -> Result<(), EngineError>;

    /// Load all swipes for the given message IDs, grouped by message_id.
    fn load_swipes_for_messages(
        &self,
        message_ids: &[u64],
    ) -> Result<HashMap<u64, Vec<Swipe>>, EngineError>;
}

pub struct SqliteMessageSwipeRepository {
    pool: DbPool,
}

impl SqliteMessageSwipeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl MessageSwipeStorage for SqliteMessageSwipeRepository {
    fn insert_swipe(
        &self,
        message_id: u64,
        swipe: &Swipe,
        index: usize,
    ) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, location_header, event_header)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                message_id as i64,
                index as i64,
                &swipe.text,
                swipe.snapshot_id.map(|id| id as i64),
                swipe.location_header.as_deref(),
                swipe.event_header.as_deref(),
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to insert swipe: {e}")))?;
        Ok(())
    }

    fn update_swipe_text(
        &self,
        message_id: u64,
        swipe_index: usize,
        text: &str,
    ) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE message_swipes SET text = ?1 WHERE message_id = ?2 AND swipe_index = ?3",
            rusqlite::params![text, message_id as i64, swipe_index as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to update swipe text: {e}")))?;
        Ok(())
    }

    fn shift_swipe_indices(&self, message_id: u64, offset: usize) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE message_swipes SET swipe_index = swipe_index + ?1 WHERE message_id = ?2",
            rusqlite::params![offset as i64, message_id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to shift swipe indices: {e}")))?;
        Ok(())
    }

    fn load_swipes_for_messages(
        &self,
        message_ids: &[u64],
    ) -> Result<HashMap<u64, Vec<Swipe>>, EngineError> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let conn = self.pool.conn();
        let placeholders = message_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT message_id, swipe_index, text, snapshot_id, location_header, event_header
             FROM message_swipes
             WHERE message_id IN ({placeholders})
             ORDER BY message_id, swipe_index"
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| EngineError::Config(format!("Failed to prepare swipe query: {e}")))?;

        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(message_ids.iter().map(|id| *id as i64)),
                |row| {
                    Ok((
                        row.get::<_, i64>(0)? as u64,
                        Swipe {
                            text: row.get(2)?,
                            snapshot_id: row.get::<_, Option<i64>>(3)?.map(|id| id as u64),
                            location_header: row.get(4)?,
                            event_header: row.get(5)?,
                        },
                    ))
                },
            )
            .map_err(|e| EngineError::Config(format!("Failed to query swipes: {e}")))?;

        let mut result: HashMap<u64, Vec<Swipe>> = HashMap::new();
        for row in rows {
            let (message_id, swipe) =
                row.map_err(|e| EngineError::Config(format!("Failed to read swipe row: {e}")))?;
            result.entry(message_id).or_default().push(swipe);
        }

        Ok(result)
    }
}
