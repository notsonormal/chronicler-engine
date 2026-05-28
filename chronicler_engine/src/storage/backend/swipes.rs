use std::collections::HashMap;

use crate::error::EngineError;
use crate::model::message::Swipe;
use crate::storage::backend::{Backend, Operation, Storage};

impl Storage {
    pub fn insert_swipe(
        &self,
        message_id: u64,
        swipe: &Swipe,
        index: usize,
    ) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::InsertSwipe, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
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
            Backend::InMemory(data) => {
                let entry = data.swipes.entry(message_id).or_default();
                if index < entry.len() {
                    entry.insert(index, swipe.clone());
                } else {
                    entry.push(swipe.clone());
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn update_swipe_text(
        &self,
        message_id: u64,
        swipe_index: usize,
        text: &str,
    ) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::UpdateSwipeText, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE message_swipes SET text = ?1 WHERE message_id = ?2 AND swipe_index = ?3",
                    rusqlite::params![text, message_id as i64, swipe_index as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to update swipe text: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(swipes) = data.swipes.get_mut(&message_id) {
                    if let Some(swipe) = swipes.get_mut(swipe_index) {
                        swipe.text = text.to_string();
                    }
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn shift_swipe_indices(&self, message_id: u64, offset: usize) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::ShiftSwipeIndices, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE message_swipes SET swipe_index = swipe_index + ?1 WHERE message_id = ?2",
                    rusqlite::params![offset as i64, message_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to shift swipe indices: {e}")))?;
                Ok(())
            }
            Backend::InMemory(_) => {
                // No-op for in-memory: insert_swipe handles index insertion directly.
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn load_swipes_for_messages(
        &self,
        message_ids: &[u64],
    ) -> Result<HashMap<u64, Vec<Swipe>>, EngineError> {
        self.with_backend_mut(Operation::LoadSwipesForMessages, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                if message_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let conn = pool.conn();
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
            Backend::InMemory(data) => {
                let mut result = HashMap::new();
                for &id in message_ids {
                    if let Some(s) = data.swipes.get(&id) {
                        result.insert(id, s.clone());
                    }
                }
                Ok(result)
            }
            Backend::Test { .. } => unreachable!(),
        })
    }
}
