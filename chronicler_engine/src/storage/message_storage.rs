use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::EngineError;
use crate::model::message::Message;
use crate::storage::db::DbPool;
use crate::storage::models::message::DbMessage;

pub trait MessageStorage: Send + Sync {
    fn set_game_id(&self, game_id: u64);
    fn current_game_id(&self) -> u64;
    fn insert_message(&self, msg: &Message) -> Result<u64, EngineError>;
    fn delete_message(&self, id: u64) -> Result<(), EngineError>;

    /// Load message rows without swipe data attached.
    fn load_message_rows(&self) -> Result<Vec<Message>, EngineError>;
    /// Get the active swipe index for a message.
    fn get_active_swipe_index(&self, id: u64) -> Result<usize, EngineError>;
    /// Update the active swipe index for a message.
    fn update_active_swipe(&self, message_id: u64, index: usize) -> Result<(), EngineError>;

    /// Soft-delete a message (used by retry so it can be restored on failure).
    fn soft_delete_message(&self, id: u64) -> Result<(), EngineError>;
    fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError>;
    /// Hard-delete soft-deleted messages (CASCADE removes their swipes).
    fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError>;
}

pub struct SqliteMessageRepository {
    pool: DbPool,
    game_id: AtomicU64,
}

impl SqliteMessageRepository {
    pub fn new(pool: DbPool, game_id: u64) -> Self {
        Self {
            pool,
            game_id: AtomicU64::new(game_id),
        }
    }

    fn game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }
}

impl MessageStorage for SqliteMessageRepository {
    fn set_game_id(&self, game_id: u64) {
        self.game_id.store(game_id, Ordering::SeqCst);
    }

    fn current_game_id(&self) -> u64 {
        self.game_id()
    }

    fn insert_message(&self, msg: &Message) -> Result<u64, EngineError> {
        let conn = self.pool.conn();
        let db_msg =
            crate::storage::mappers::message::model_message_to_db(msg, self.game_id() as i64)?;
        conn.execute(
            "INSERT INTO messages (game_id, sender, log_type, timestamp, active_swipe_index, is_deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                db_msg.game_id,
                db_msg.sender.as_deref(),
                db_msg.log_type_json,
                db_msg.timestamp,
                db_msg.active_swipe_index,
                db_msg.is_deleted,
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to insert message: {e}")))?;
        let id = conn.last_insert_rowid() as u64;
        Ok(id)
    }

    fn delete_message(&self, id: u64) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
            rusqlite::params![id as i64, self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete message: {e}")))?;
        Ok(())
    }

    fn load_message_rows(&self) -> Result<Vec<Message>, EngineError> {
        let conn = self.pool.conn();
        let game_id = self.game_id() as i64;

        let mut stmt = conn
            .prepare(
                "SELECT id, sender, log_type, timestamp, active_swipe_index
                 FROM messages
                 WHERE game_id = ?1 AND is_deleted = 0
                 ORDER BY id ASC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare message query: {e}")))?;

        let msg_rows = stmt
            .query_map(rusqlite::params![game_id], |row| {
                Ok(DbMessage {
                    id: row.get(0)?,
                    game_id,
                    sender: row.get(1)?,
                    log_type_json: row.get(2)?,
                    timestamp: row.get(3)?,
                    active_swipe_index: row.get(4)?,
                    is_deleted: 0,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to query messages: {e}")))?;

        let mut messages: Vec<Message> = Vec::new();
        for row in msg_rows {
            let db_msg =
                row.map_err(|e| EngineError::Config(format!("Failed to read message row: {e}")))?;
            messages.push(crate::storage::mappers::message::db_message_to_model(
                &db_msg,
                &[],
            )?);
        }

        Ok(messages)
    }

    fn get_active_swipe_index(&self, id: u64) -> Result<usize, EngineError> {
        let conn = self.pool.conn();
        let idx: i64 = conn
            .query_row(
                "SELECT active_swipe_index FROM messages WHERE id = ?1 AND game_id = ?2",
                rusqlite::params![id as i64, self.game_id() as i64],
                |row| row.get(0),
            )
            .map_err(|e| EngineError::Config(format!("Failed to get active swipe index: {e}")))?;
        Ok(idx as usize)
    }

    fn update_active_swipe(&self, message_id: u64, index: usize) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE messages SET active_swipe_index = ?1 WHERE id = ?2 AND game_id = ?3",
            rusqlite::params![index as i64, message_id as i64, self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to update active swipe: {e}")))?;
        Ok(())
    }

    fn soft_delete_message(&self, id: u64) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE messages SET is_deleted = 1 WHERE id = ?1 AND game_id = ?2",
            rusqlite::params![id as i64, self.game_id() as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to soft delete message: {e}")))?;
        Ok(())
    }

    fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        for id in ids {
            conn.execute(
                "UPDATE messages SET is_deleted = 0 WHERE id = ?1 AND game_id = ?2",
                rusqlite::params![*id as i64, self.game_id() as i64],
            )
            .map_err(|e| EngineError::Config(format!("Failed to restore message: {e}")))?;
        }
        Ok(())
    }

    fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        for id in ids {
            conn.execute(
                "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
                rusqlite::params![*id as i64, self.game_id() as i64],
            )
            .map_err(|e| EngineError::Config(format!("Failed to purge message: {e}")))?;
        }
        Ok(())
    }
}
