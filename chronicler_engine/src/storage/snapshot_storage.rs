use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;

use crate::error::EngineError;
use crate::model::checkpoint::Checkpoint;
use crate::model::message::Message;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::db::DbPool;
use crate::storage::message_storage::MessageStorage;

fn parse_json<T: DeserializeOwned>(col: usize, json: &str) -> Result<T, rusqlite::Error> {
    serde_json::from_str(json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn parse_datetime(col: usize, s: &str) -> Result<DateTime<Utc>, rusqlite::Error> {
    Ok(DateTime::parse_from_rfc3339(s)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc))
}

pub trait SnapshotStorage: Send + Sync {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError>;
    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn commit(&self, snapshot_id: u64) -> Result<(), EngineError>;
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
        let movement_json = serde_json::to_string(&snapshot.movement)
            .map_err(|e| EngineError::Config(format!("Failed to serialize movement: {e}")))?;
        let narrative_json = serde_json::to_string(&snapshot.narrative)
            .map_err(|e| EngineError::Config(format!("Failed to serialize narrative: {e}")))?;
        let scene_json = serde_json::to_string(&snapshot.scene)
            .map_err(|e| EngineError::Config(format!("Failed to serialize scene: {e}")))?;
        let character_state_json =
            serde_json::to_string(&snapshot.character_state).map_err(|e| {
                EngineError::Config(format!("Failed to serialize character_state: {e}"))
            })?;

        conn.execute(
            "INSERT INTO game_state_snapshots
             (game_id, movement, narrative, scene, character_state, committed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                self.game_id as i64,
                movement_json,
                narrative_json,
                scene_json,
                character_state_json,
                snapshot.committed as i32,
                snapshot.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to save snapshot: {e}")))?;

        Ok(conn.last_insert_rowid() as u64)
    }

    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, movement, narrative, scene, character_state, committed, created_at
                 FROM game_state_snapshots
                 WHERE game_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        match stmt.query_row(rusqlite::params![self.game_id as i64], row_to_snapshot) {
            Ok(snapshot) => Ok(Some(snapshot)),
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
                "SELECT id, movement, narrative, scene, character_state, committed, created_at
                 FROM game_state_snapshots
                 WHERE id = ?1 AND game_id = ?2
                 ORDER BY id DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        match stmt.query_row(rusqlite::params![id, self.game_id as i64], row_to_snapshot) {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load snapshot by id: {e}"
            ))),
        }
    }

    fn commit(&self, snapshot_id: u64) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE game_state_snapshots SET committed = 1 WHERE id = ?1 AND game_id = ?2",
            rusqlite::params![snapshot_id, self.game_id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to commit snapshot: {e}")))?;
        Ok(())
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
        conn.execute(
            "INSERT INTO checkpoints (id, snapshot_id, name, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 snapshot_id=excluded.snapshot_id,
                 name=excluded.name,
                 created_at=excluded.created_at",
            rusqlite::params![
                checkpoint.id,
                checkpoint.snapshot_id,
                checkpoint.name,
                checkpoint.created_at.to_rfc3339(),
            ],
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

        match stmt.query_row(rusqlite::params![id], row_to_checkpoint) {
            Ok(cp) => Ok(Some(cp)),
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
            .query_map([], row_to_checkpoint)
            .map_err(|e| EngineError::Config(format!("Failed to list checkpoints: {e}")))?;

        rows.map(|row| {
            row.map_err(|e| EngineError::Config(format!("Failed to read checkpoint row: {e}")))
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
        let log_type_str = serde_json::to_string(&msg.log_type)
            .map_err(|e| EngineError::Config(format!("Failed to serialize log_type: {e}")))?;
        conn.execute(
            "INSERT INTO messages (game_id, sender, text, log_type, timestamp, location_header, event_header, snapshot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                self.game_id as i64,
                msg.sender.as_deref(),
                &msg.text,
                log_type_str,
                msg.timestamp.to_rfc3339(),
                msg.location_header.as_deref(),
                msg.event_header.as_deref(),
                msg.snapshot_id.map(|id| id as i64),
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
                let log_type_str: String = row.get(3)?;
                let log_type = serde_json::from_str(&log_type_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                let timestamp_str: String = row.get(4)?;
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(Message {
                    id: row.get::<_, i64>(0)? as u64,
                    sender: row.get(1)?,
                    text: row.get(2)?,
                    log_type,
                    timestamp,
                    location_header: row.get(5)?,
                    event_header: row.get(6)?,
                    snapshot_id: row.get::<_, Option<i64>>(7)?.map(|id| id as u64),
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to query messages: {e}")))?;

        rows.map(|row| {
            row.map_err(|e| EngineError::Config(format!("Failed to read message row: {e}")))
        })
        .collect()
    }
}

fn row_to_snapshot(row: &rusqlite::Row) -> Result<GameStateSnapshot, rusqlite::Error> {
    let movement_json: String = row.get(1)?;
    let narrative_json: String = row.get(2)?;
    let scene_json: String = row.get(3)?;
    let character_state_json: String = row.get(4)?;
    let created_at_str: String = row.get(6)?;

    let movement = parse_json(1, &movement_json)?;
    let narrative = parse_json(2, &narrative_json)?;
    let scene = parse_json(3, &scene_json)?;
    let character_state = parse_json(4, &character_state_json)?;
    let created_at = parse_datetime(6, &created_at_str)?;

    Ok(GameStateSnapshot {
        db_id: Some(row.get::<_, i64>(0)? as u64),
        movement,
        narrative,
        scene,
        character_state,
        committed: row.get::<_, i32>(5)? != 0,
        created_at,
    })
}

fn row_to_checkpoint(row: &rusqlite::Row) -> Result<Checkpoint, rusqlite::Error> {
    let created_at_str: String = row.get(3)?;
    let created_at = parse_datetime(3, &created_at_str)?;

    Ok(Checkpoint {
        id: row.get(0)?,
        snapshot_id: row.get::<_, i64>(1)? as u64,
        name: row.get(2)?,
        created_at,
    })
}
