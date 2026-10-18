//! [DOC: docs/diataxis/reference/game_flow.md]
//! Swipe data storage

use std::collections::HashMap;

use crate::error::EngineError;
use crate::domain::model::message::Swipe;
use crate::adapters::driven::storage::models::swipe::DbSwipe;
use crate::adapters::driven::storage::utils::to_json;
use crate::adapters::driven::storage::{Backend, Storage};

impl Storage {
    pub fn insert_swipe(
        &self,
        message_id: u64,
        swipe: &Swipe,
        index: usize,
    ) -> Result<(), EngineError> {
        self.with_backend_mut("insert_swipe", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, room_report, event_header, impersonated, guide_text, impersonate_direction, state, maker)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    rusqlite::params![
                        message_id as i64,
                        index as i64,
                        &swipe.text,
                        swipe.snapshot_id.map(|id| id as i64),
                        swipe.room_report.as_deref(),
                        swipe.event_header.as_deref(),
                        if swipe.impersonated { 1 } else { 0 },
                        swipe.guide_text.as_deref(),
                        swipe.impersonate_direction.as_deref(),
                        to_json("swipe state", swipe.state.as_ref())?,
                        to_json("swipe maker", swipe.maker.as_ref())?,
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
        })
    }

    pub fn update_swipe_text(
        &self,
        message_id: u64,
        swipe_index: usize,
        text: &str,
    ) -> Result<(), EngineError> {
        self.with_backend_mut("update_swipe_text", |backend| match backend {
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
                data.update_swipe_text(message_id, swipe_index, text);
                Ok(())
            }
        })
    }

    pub fn update_swipe_room_report(
        &self,
        message_id: u64,
        swipe_index: usize,
        room_report: &str,
    ) -> Result<(), EngineError> {
        self.with_backend_mut("update_swipe_room_report", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE message_swipes SET room_report = ?1 WHERE message_id = ?2 AND swipe_index = ?3",
                    rusqlite::params![room_report, message_id as i64, swipe_index as i64],
                )
                .map_err(|e| {
                    EngineError::Config(format!("Failed to update swipe location header: {e}"))
                })?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.update_swipe_room_report(message_id, swipe_index, room_report);
                Ok(())
            }
        })
    }

    pub fn shift_swipe_indices(&self, message_id: u64, offset: usize) -> Result<(), EngineError> {
        self.with_backend_mut("shift_swipe_indices", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE message_swipes SET swipe_index = swipe_index + ?1 WHERE message_id = ?2",
                    rusqlite::params![offset as i64, message_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to shift swipe indices: {e}")))?;
                Ok(())
            }
            Backend::InMemory(_) => Ok(()),
        })
    }

    pub fn load_swipes_for_messages(
        &self,
        message_ids: &[u64],
    ) -> Result<HashMap<u64, Vec<Swipe>>, EngineError> {
        self.with_backend_mut("load_swipes_for_messages", |backend| match backend {
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
                    "SELECT message_id, swipe_index, text, snapshot_id, room_report, event_header, impersonated, guide_text, impersonate_direction, state, maker
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
                                DbSwipe {
                                    id: 0,
                                    message_id: row.get(0)?,
                                    swipe_index: row.get(1)?,
                                    text: row.get(2)?,
                                    snapshot_id: row.get(3)?,
                                    room_report: row.get(4)?,
                                    event_header: row.get(5)?,
                                    impersonated: row.get(6)?,
                                    guide_text: row.get(7)?,
                                    impersonate_direction: row.get(8)?,
                                    state_json: row.get(9)?,
                                    maker_json: row.get(10)?,
                                },
                            ))
                        },
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to query swipes: {e}")))?;

                let mut result: HashMap<u64, Vec<Swipe>> = HashMap::new();
                for row in rows {
                    let (message_id, db_swipe) =
                        row.map_err(|e| EngineError::Config(format!("Failed to read swipe row: {e}")))?;
                    result
                        .entry(message_id)
                        .or_default()
                        .push(Swipe::try_from(&db_swipe)?);
                }

                Ok(result)
            }
            Backend::InMemory(data) => {
                Ok(data.load_swipes_for_messages(message_ids))
            }
        })
    }

    pub fn count_swipes_for_message(&self, message_id: u64) -> Result<usize, EngineError> {
        self.with_backend_mut("count_swipes_for_message", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM message_swipes WHERE message_id = ?1",
                        rusqlite::params![message_id as i64],
                        |row| row.get(0),
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to count swipes: {e}")))?;
                Ok(count as usize)
            }
            Backend::InMemory(data) => {
                let count = data
                    .swipes
                    .get(&message_id)
                    .map(|vec| vec.len())
                    .unwrap_or(0);
                Ok(count)
            }
        })
    }

    /// Per-swipe errors are logged and swallowed.
    pub fn persist_swipes(&self, msg_id: u64, swipes: &[Swipe]) {
        for (index, swipe) in swipes.iter().enumerate() {
            if let Err(e) = self.insert_swipe(msg_id, swipe, index) {
                tracing::error!("persist_swipes: swipe {index} failed: {e}");
            }
        }
    }
}
