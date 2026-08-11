//! [DOC: docs/diataxis/reference/storage.md]
//! Snapshot storage operations

use crate::error::EngineError;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::adapters::driven::storage::backend::{Backend, Storage};
use crate::adapters::driven::storage::models::game_state_snapshot::DbGameStateSnapshot;

impl Storage {
    pub fn save_snapshot(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("save_snapshot", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_snap =
                    crate::adapters::driven::storage::mappers::state_snapshot::snapshot_to_db(
                        snapshot,
                        game_id as i64,
                    )?;

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
            Backend::InMemory(data) => {
                let id = data.next_snapshot_id;
                data.next_snapshot_id += 1;
                let mut snap = snapshot.clone();
                snap.db_id = Some(id);
                data.snapshots.entry(game_id).or_default().push(snap);
                Ok(id)
            }
        })
    }

    pub fn load_latest_snapshot(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("load_latest_snapshot", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, game_id, movement, narrative, scene, npc_encounter_log, created_at
                         FROM game_state_snapshots
                         WHERE game_id = ?1
                         ORDER BY created_at DESC, id DESC
                         LIMIT 1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let db_result = stmt.query_row(rusqlite::params![game_id as i64], |row| {
                    DbGameStateSnapshot::from_row(row)
                });

                match db_result {
                    Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!(
                        "Failed to load latest snapshot: {e}"
                    ))),
                }
            }
            Backend::InMemory(data) => {
                let result = data.snapshots.get(&game_id).and_then(|vec| {
                    vec.iter()
                        .max_by(|a, b| {
                            a.created_at
                                .cmp(&b.created_at)
                                .then_with(|| a.db_id.cmp(&b.db_id))
                        })
                        .cloned()
                });
                Ok(result)
            }
        })
    }

    pub fn load_snapshot_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError> {
        let game_id = self.game_id();
        self.with_backend_mut("load_snapshot_by_id", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, game_id, movement, narrative, scene, npc_encounter_log, created_at
                         FROM game_state_snapshots
                         WHERE id = ?1 AND game_id = ?2",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let db_result = stmt.query_row(rusqlite::params![id, game_id as i64], |row| {
                    DbGameStateSnapshot::from_row(row)
                });

                match db_result {
                    Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!(
                        "Failed to load snapshot by id: {e}"
                    ))),
                }
            }
            Backend::InMemory(data) => {
                let result = data
                    .snapshots
                    .get(&game_id)
                    .and_then(|vec| vec.iter().find(|s| s.db_id == Some(id)).cloned());
                Ok(result)
            }
        })
    }
}
