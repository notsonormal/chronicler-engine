//! [DOC: docs/system/game_flow.md]
//! Message storage operations

use crate::error::EngineError;
use crate::model::message::Message;
use crate::storage::backend::{Backend, Storage};
use crate::storage::models::message::DbMessage;

impl Storage {
    pub fn insert_message(&self, msg: &Message) -> Result<u64, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("insert_message", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_msg =
                    crate::storage::mappers::message::model_message_to_db(msg, game_id as i64)?;
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
            Backend::Test { .. } => unreachable!(),
            Backend::InMemory(data) => {
                data.next_message_id += 1;
                let id = data.next_message_id;
                let mut msg = msg.clone();
                msg.id = id;
                msg.swipes.clear();
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
            Backend::Test { .. } => unreachable!(),
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
                        "SELECT id, sender, message_type, timestamp, active_swipe_index
                         FROM messages
                         WHERE game_id = ?1 AND is_deleted = 0
                         ORDER BY id ASC",
                    )
                    .map_err(|e| {
                        EngineError::Config(format!("Failed to prepare message query: {e}"))
                    })?;

                let msg_rows = stmt
                    .query_map(rusqlite::params![game_id as i64], |row| {
                        Ok(DbMessage {
                            id: row.get(0)?,
                            game_id: game_id as i64,
                            sender: row.get(1)?,
                            message_type_json: row.get(2)?,
                            timestamp: row.get(3)?,
                            active_swipe_index: row.get(4)?,
                            is_deleted: 0,
                        })
                    })
                    .map_err(|e| EngineError::Config(format!("Failed to query messages: {e}")))?;

                let mut messages: Vec<Message> = Vec::new();
                for row in msg_rows {
                    let db_msg = row.map_err(|e| {
                        EngineError::Config(format!("Failed to read message row: {e}"))
                    })?;
                    messages.push(crate::storage::mappers::message::db_message_to_model(
                        &db_msg,
                        &[],
                    )?);
                }

                Ok(messages)
            }
            Backend::InMemory(data) => Ok(data
                .messages
                .get(&game_id)
                .map(|vec| vec.iter().filter(|m| !m.is_deleted).cloned().collect())
                .unwrap_or_default()),
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn get_active_swipe_index(&self, id: u64) -> Result<usize, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("get_active_swipe_index", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let idx: i64 = conn
                    .query_row(
                        "SELECT active_swipe_index FROM messages WHERE id = ?1 AND game_id = ?2",
                        rusqlite::params![id as i64, game_id as i64],
                        |row| row.get(0),
                    )
                    .map_err(|e| {
                        EngineError::Config(format!("Failed to get active swipe index: {e}"))
                    })?;
                Ok(idx as usize)
            }
            Backend::InMemory(data) => {
                if let Some(vec) = data.messages.get(&game_id) {
                    if let Some(m) = vec.iter().find(|m| m.id == id) {
                        return Ok(m.active_swipe_index);
                    }
                }
                Err(EngineError::Config(format!("Message {id} not found")))
            }
            Backend::Test { .. } => unreachable!(),
        })
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
                if let Some(vec) = data.messages.get_mut(&game_id) {
                    if let Some(m) = vec.iter_mut().find(|m| m.id == message_id) {
                        m.active_swipe_index = index;
                    }
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
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
                if let Some(vec) = data.messages.get_mut(&game_id) {
                    if let Some(m) = vec.iter_mut().find(|m| m.id == id) {
                        m.is_deleted = true;
                    }
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
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
                if let Some(vec) = data.messages.get_mut(&game_id) {
                    for m in vec.iter_mut().filter(|m| ids.contains(&m.id)) {
                        m.is_deleted = false;
                    }
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
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
            Backend::Test { .. } => unreachable!(),
        })
    }
}
