use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::db::DbPool;

pub trait SnapshotStorage: Send + Sync {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), EngineError>;
    fn load_latest(
        &self,
        message_id: Option<&str>,
    ) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn load_by_message(
        &self,
        message_id: &str,
        swipe_index: u32,
    ) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn commit(&self, snapshot_id: &str) -> Result<(), EngineError>;
    fn reset(&self) -> Result<(), EngineError>;
}

pub struct SqliteSnapshotStorage {
    pool: DbPool,
}

impl SqliteSnapshotStorage {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl SnapshotStorage for SqliteSnapshotStorage {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), EngineError> {
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
             (id, message_id, swipe_index, movement, narrative, scene, character_state, committed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(message_id, swipe_index) DO UPDATE SET
                 id=excluded.id,
                 movement=excluded.movement,
                 narrative=excluded.narrative,
                 scene=excluded.scene,
                 character_state=excluded.character_state,
                 committed=excluded.committed,
                 created_at=excluded.created_at",
            rusqlite::params![
                snapshot.id,
                snapshot.message_id,
                snapshot.swipe_index,
                movement_json,
                narrative_json,
                scene_json,
                character_state_json,
                snapshot.committed as i32,
                snapshot.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to save snapshot: {e}")))?;

        Ok(())
    }

    fn load_latest(
        &self,
        message_id: Option<&str>,
    ) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = if message_id.is_some() {
            conn.prepare(
                "SELECT id, message_id, swipe_index, movement, narrative, scene, character_state, committed, created_at
                 FROM game_state_snapshots
                 WHERE message_id = ?1
                 ORDER BY created_at DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?
        } else {
            conn.prepare(
                "SELECT id, message_id, swipe_index, movement, narrative, scene, character_state, committed, created_at
                 FROM game_state_snapshots
                 ORDER BY created_at DESC
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?
        };

        let row = if let Some(msg_id) = message_id {
            stmt.query_row(rusqlite::params![msg_id], row_to_snapshot)
        } else {
            stmt.query_row([], row_to_snapshot)
        };

        match row {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load latest snapshot: {e}"
            ))),
        }
    }

    fn load_by_message(
        &self,
        message_id: &str,
        swipe_index: u32,
    ) -> Result<Option<GameStateSnapshot>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, message_id, swipe_index, movement, narrative, scene, character_state, committed, created_at
                 FROM game_state_snapshots
                 WHERE message_id = ?1 AND swipe_index = ?2
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        match stmt.query_row(rusqlite::params![message_id, swipe_index], row_to_snapshot) {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!(
                "Failed to load snapshot by message: {e}"
            ))),
        }
    }

    fn commit(&self, snapshot_id: &str) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "UPDATE game_state_snapshots SET committed = 1 WHERE id = ?1",
            rusqlite::params![snapshot_id],
        )
        .map_err(|e| EngineError::Config(format!("Failed to commit snapshot: {e}")))?;
        Ok(())
    }

    fn reset(&self) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute("DELETE FROM game_state_snapshots", [])
            .map_err(|e| EngineError::Config(format!("Failed to reset snapshots: {e}")))?;
        Ok(())
    }
}

fn row_to_snapshot(row: &rusqlite::Row) -> Result<GameStateSnapshot, rusqlite::Error> {
    let movement_json: String = row.get(3)?;
    let narrative_json: String = row.get(4)?;
    let scene_json: String = row.get(5)?;
    let character_state_json: String = row.get(6)?;
    let created_at_str: String = row.get(8)?;

    let movement = serde_json::from_str(&movement_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let narrative = serde_json::from_str(&narrative_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let scene = serde_json::from_str(&scene_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let character_state = serde_json::from_str(&character_state_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);

    Ok(GameStateSnapshot {
        id: row.get(0)?,
        message_id: row.get(1)?,
        swipe_index: row.get(2)?,
        movement,
        narrative,
        scene,
        character_state,
        committed: row.get::<_, i32>(7)? != 0,
        created_at,
    })
}
