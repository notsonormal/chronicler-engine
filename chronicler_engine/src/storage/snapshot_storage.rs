use crate::error::EngineError;
use crate::model::checkpoint::Checkpoint;
use crate::model::message::Message;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::db::DbPool;
use crate::storage::message_storage::MessageStorage;
use crate::storage::models::checkpoint::DbCheckpoint;
use crate::storage::models::game_state_snapshot::DbGameStateSnapshot;
use crate::storage::models::message::DbMessage;

pub trait SnapshotStorage: Send + Sync {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError>;
    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn reset(&self) -> Result<(), EngineError>;

    fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), EngineError>;
    fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, EngineError>;
    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, EngineError>;
    fn delete_checkpoint(&self, id: &str) -> Result<(), EngineError>;
}

pub struct SqliteGameStorage {
    pool: DbPool,
    game_id: u64,
}

impl SqliteGameStorage {
    pub fn new(pool: DbPool, game_id: u64) -> Self {
        Self { pool, game_id }
    }
}

impl SnapshotStorage for SqliteGameStorage {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
        let conn = self.pool.conn();
        let db_snap =
            crate::storage::mappers::state_snapshot::snapshot_to_db(snapshot, self.game_id as i64)?;

        conn.execute(
            "INSERT INTO game_state_snapshots
             (game_id, movement, narrative, scene, npc_encounter_log, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                db_snap.game_id,
                db_snap.movement_json,
                db_snap.narrative_json,
                db_snap.scene_json,
                db_snap.npc_encounter_log_json,
                db_snap.created_at,
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to save snapshot: {e}")))?;

        Ok(conn.last_insert_rowid() as u64)
    }

    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, movement, narrative, scene, npc_encounter_log, created_at
                 FROM game_state_snapshots
                 WHERE game_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![self.game_id as i64], |row| {
            Ok(DbGameStateSnapshot {
                id: row.get(0)?,
                game_id: self.game_id as i64,
                movement_json: row.get(1)?,
                narrative_json: row.get(2)?,
                scene_json: row.get(3)?,
                npc_encounter_log_json: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match db_result {
            Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load latest snapshot: {e}"
            ))),
        }
    }

    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, movement, narrative, scene, npc_encounter_log, created_at
                 FROM game_state_snapshots
                 WHERE id = ?1 AND game_id = ?2
                 ORDER BY id DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![id, self.game_id as i64], |row| {
            Ok(DbGameStateSnapshot {
                id: row.get(0)?,
                game_id: self.game_id as i64,
                movement_json: row.get(1)?,
                narrative_json: row.get(2)?,
                scene_json: row.get(3)?,
                npc_encounter_log_json: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match db_result {
            Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load snapshot by id: {e}"
            ))),
        }
    }

    fn reset(&self) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        // Delete checkpoints first (they reference snapshots).
        // Checkpoints table currently lacks game_id, so we clear all checkpoints.
        conn.execute("DELETE FROM checkpoints", [])
            .map_err(|e| EngineError::Config(format!("Failed to reset checkpoints: {e}")))?;
        conn.execute(
            "DELETE FROM game_state_snapshots WHERE game_id = ?1",
            rusqlite::params![self.game_id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to reset snapshots: {e}")))?;
        conn.execute(
            "DELETE FROM messages WHERE game_id = ?1",
            rusqlite::params![self.game_id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to reset messages: {e}")))?;
        Ok(())
    }

    fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        let db_cp = DbCheckpoint::from(checkpoint);
        conn.execute(
            "INSERT INTO checkpoints (id, snapshot_id, name, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 snapshot_id=excluded.snapshot_id,
                 name=excluded.name,
                 created_at=excluded.created_at",
            rusqlite::params![db_cp.id, db_cp.snapshot_id, db_cp.name, db_cp.created_at,],
        )
        .map_err(|e| EngineError::Config(format!("Failed to save checkpoint: {e}")))?;
        Ok(())
    }

    fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, snapshot_id, name, created_at
                 FROM checkpoints
                 WHERE id = ?1
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![id], |row| {
            Ok(DbCheckpoint {
                id: row.get(0)?,
                snapshot_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
            })
        });

        match db_result {
            Ok(db_cp) => Ok(Some(Checkpoint::try_from(&db_cp)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load checkpoint: {e}"
            ))),
        }
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, snapshot_id, name, created_at
                 FROM checkpoints
                 ORDER BY created_at DESC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(DbCheckpoint {
                    id: row.get(0)?,
                    snapshot_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to list checkpoints: {e}")))?;

        rows.map(|row| {
            let db_cp = row
                .map_err(|e| EngineError::Config(format!("Failed to read checkpoint row: {e}")))?;
            Checkpoint::try_from(&db_cp)
        })
        .collect()
    }

    fn delete_checkpoint(&self, id: &str) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "DELETE FROM checkpoints WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete checkpoint: {e}")))?;
        Ok(())
    }
}

impl MessageStorage for SqliteGameStorage {
    fn insert_message(&self, msg: &mut Message) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        let db_msg = crate::storage::mappers::message::message_to_db(msg, self.game_id as i64)?;
        conn.execute(
            "INSERT INTO messages (game_id, sender, text, log_type, timestamp, location_header, event_header, snapshot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                db_msg.game_id,
                db_msg.sender.as_deref(),
                db_msg.text,
                db_msg.log_type_json,
                db_msg.timestamp,
                db_msg.location_header.as_deref(),
                db_msg.event_header.as_deref(),
                db_msg.snapshot_id,
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to insert message: {e}")))?;
        msg.id = conn.last_insert_rowid() as u64;
        Ok(())
    }

    fn update_message(&self, id: u64, text: &str) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE messages SET text = ?1 WHERE id = ?2 AND game_id = ?3",
            rusqlite::params![text, id as i64, self.game_id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to update message: {e}")))?;
        Ok(())
    }

    fn delete_message(&self, id: u64) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
            rusqlite::params![id as i64, self.game_id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete message: {e}")))?;
        Ok(())
    }

    fn load_messages(&self) -> Result<Vec<Message>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, sender, text, log_type, timestamp, location_header, event_header, snapshot_id
                 FROM messages
                 WHERE game_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare message query: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![self.game_id as i64], |row| {
                Ok(DbMessage {
                    id: row.get(0)?,
                    game_id: self.game_id as i64,
                    sender: row.get(1)?,
                    text: row.get(2)?,
                    log_type_json: row.get(3)?,
                    timestamp: row.get(4)?,
                    location_header: row.get(5)?,
                    event_header: row.get(6)?,
                    snapshot_id: row.get(7)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to query messages: {e}")))?;

        rows.map(|row| {
            let db_msg =
                row.map_err(|e| EngineError::Config(format!("Failed to read message row: {e}")))?;
            Message::try_from(&db_msg)
        })
        .collect()
    }
}
