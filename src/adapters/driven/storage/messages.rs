//! [DOC: docs/diataxis/reference/game_flow.md]
//! Message storage operations

use crate::error::EngineError;
use crate::domain::model::message::Message;
use crate::adapters::driven::storage::{Backend, Storage};
use crate::adapters::driven::storage::models::message::{DbMessage, DbSwipe};

impl Storage {
    pub fn insert_message(&self, msg: &Message) -> Result<u64, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("insert_message", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_msg = DbMessage::try_from((msg, game_id as i64))?;
                conn.execute(
                    "INSERT INTO messages (game_id, sender, message_type, timestamp, active_swipe_index, is_deleted)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        db_msg.game_id,
                        db_msg.sender.as_deref(),
                        db_msg.message_type_json,
                        db_msg.timestamp,
                        db_msg.active_swipe_index,
                        db_msg.is_deleted,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to insert message: {e}")))?;
                let id = conn.last_insert_rowid() as u64;
                Ok(id)
            }
            Backend::InMemory(data) => {
                data.next_message_id += 1;
                let id = data.next_message_id;
                let mut msg = msg.clone();
                msg.id = id;
                data.messages.entry(game_id).or_default().push(msg);
                Ok(id)
            }
        })
    }

    pub fn delete_message(&self, id: u64) -> Result<(), EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("delete_message", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
                    rusqlite::params![id as i64, game_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to delete message: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(vec) = data.messages.get_mut(&game_id) {
                    vec.retain(|m| m.id != id);
                }
                Ok(())
            }
        })
    }

    pub fn load_message_rows(&self) -> Result<Vec<Message>, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("load_message_rows", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, game_id, sender, message_type, timestamp, active_swipe_index
                         FROM messages
                         WHERE game_id = ?1 AND is_deleted = 0
                         ORDER BY id ASC",
                    )
                    .map_err(|e| {
                        EngineError::Config(format!("Failed to prepare message query: {e}"))
                    })?;

                let msg_rows = stmt
                    .query_map(rusqlite::params![game_id as i64], DbMessage::from_row)
                    .map_err(|e| EngineError::Config(format!("Failed to query messages: {e}")))?;

                let mut messages: Vec<Message> = Vec::new();
                for row in msg_rows {
                    let db_msg = row.map_err(|e| {
                        EngineError::Config(format!("Failed to read message row: {e}"))
                    })?;
                    messages.push(Message::try_from((&db_msg, &[] as &[DbSwipe]))?);
                }

                Ok(messages)
            }
            Backend::InMemory(data) => Ok(data
                .messages
                .get(&game_id)
                .map(|vec| vec.iter().filter(|m| !m.is_deleted).cloned().collect())
                .unwrap_or_default()),
        })
    }

    pub fn get_active_swipe_index(&self, id: u64) -> Result<Option<usize>, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("get_active_swipe_index", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                match conn.query_row(
                    "SELECT active_swipe_index FROM messages WHERE id = ?1 AND game_id = ?2",
                    rusqlite::params![id as i64, game_id as i64],
                    |row| row.get::<_, i64>(0),
                ) {
                    Ok(idx) => Ok(Some(idx as usize)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!(
                        "Failed to get active swipe index: {e}"
                    ))),
                }
            }
            Backend::InMemory(data) => {
                let Some(vec) = data.messages.get(&game_id) else {
                    return Ok(None);
                };
                match vec.iter().find(|m| m.id == id) {
                    Some(m) => Ok(Some(m.active_swipe_index)),
                    None => Ok(None),
                }
            }
        })
    }

    /// Required-read of the active swipe index for a message. Absence becomes
    /// [`EngineError::MessageNotFound`]. Optional / existence callers should
    /// stay on [`Storage::get_active_swipe_index`](Self::get_active_swipe_index).
    pub fn require_active_swipe_index(&self, id: u64) -> Result<usize, EngineError> {
        self.get_active_swipe_index(id)?
            .ok_or_else(|| EngineError::MessageNotFound(id))
    }

    pub fn update_active_swipe(&self, message_id: u64, index: usize) -> Result<(), EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("update_active_swipe", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE messages SET active_swipe_index = ?1 WHERE id = ?2 AND game_id = ?3",
                    rusqlite::params![index as i64, message_id as i64, game_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to update active swipe: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.update_active_swipe(game_id, message_id, index);
                Ok(())
            }
        })
    }

    pub fn soft_delete_message(&self, id: u64) -> Result<(), EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("soft_delete_message", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE messages SET is_deleted = 1 WHERE id = ?1 AND game_id = ?2",
                    rusqlite::params![id as i64, game_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to soft delete message: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.soft_delete_message(game_id, id);
                Ok(())
            }
        })
    }

    pub fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("restore_soft_deleted", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                for id in ids {
                    conn.execute(
                        "UPDATE messages SET is_deleted = 0 WHERE id = ?1 AND game_id = ?2",
                        rusqlite::params![*id as i64, game_id as i64],
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to restore message: {e}")))?;
                }
                Ok(())
            }
            Backend::InMemory(data) => {
                data.restore_soft_deleted(game_id, ids);
                Ok(())
            }
        })
    }

    pub fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("purge_soft_deleted", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                for id in ids {
                    conn.execute(
                        "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
                        rusqlite::params![*id as i64, game_id as i64],
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to purge message: {e}")))?;
                }
                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(vec) = data.messages.get_mut(&game_id) {
                    vec.retain(|m| !ids.contains(&m.id));
                }
                Ok(())
            }
        })
    }

    pub fn load_messages_with_swipes(&self) -> Result<Vec<Message>, EngineError> {
        let mut messages = self.load_message_rows()?;
        let ids: Vec<u64> = messages.iter().map(|m| m.id).collect();
        let swipes_map = self.load_swipes_for_messages(&ids)?;
        for msg in &mut messages {
            if let Some(swipes) = swipes_map.get(&msg.id) {
                msg.swipes = swipes.clone();
                let fallback_applied = msg.ensure_valid_swipe_index();
                if fallback_applied {
                    tracing::warn!(
                        "active_swipe_index was out of bounds for message {} ({} swipes), fell back to 0",
                        msg.id,
                        msg.swipes.len()
                    );
                }
                msg.set_active_swipe(msg.active_swipe_index);
            }
        }
        Ok(messages)
    }
}
