use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::EngineError;
use crate::model::game::Game;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::db::DbPool;
use crate::storage::models::game::DbGame;
use crate::storage::models::game_state_snapshot::DbGameStateSnapshot;

pub trait SnapshotStorage: Send + Sync {
    fn set_game_id(&self, game_id: u64);
    fn current_game_id(&self) -> u64;
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError>;
    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError>;
    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError>;

    fn list_games(&self) -> Result<Vec<Game>, EngineError>;
    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError>;
    fn delete_game(&self, id: u64) -> Result<(), EngineError>;
    fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError>;
}

pub struct SqliteSnapshotRepository {
    pool: DbPool,
    game_id: AtomicU64,
}

impl SqliteSnapshotRepository {
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

fn db_game_to_game(db: &DbGame) -> Result<Game, EngineError> {
    Ok(Game {
        id: db.id as u64,
        world_name: db.world_name.clone(),
        name: db.name.clone(),
        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
            .map_err(|e| EngineError::Config(format!("Invalid created_at: {e}")))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&db.updated_at)
            .map_err(|e| EngineError::Config(format!("Invalid updated_at: {e}")))?
            .with_timezone(&chrono::Utc),
    })
}

impl SnapshotStorage for SqliteSnapshotRepository {
    fn set_game_id(&self, game_id: u64) {
        let current = self.game_id();
        if current != game_id {
            let conn = self.pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute(
                "UPDATE games SET updated_at = ?1 WHERE id = ?2",
                rusqlite::params![&now, game_id as i64],
            );
            self.game_id.store(game_id, Ordering::SeqCst);
        }
    }

    fn current_game_id(&self) -> u64 {
        self.game_id()
    }

    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
        let conn = self.pool.conn();
        let db_snap = crate::storage::mappers::state_snapshot::snapshot_to_db(
            snapshot,
            self.game_id() as i64,
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

        let db_result = stmt.query_row(rusqlite::params![self.game_id() as i64], |row| {
            Ok(DbGameStateSnapshot {
                id: row.get(0)?,
                game_id: self.game_id() as i64,
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
                 WHERE id = ?1 AND game_id = ?2",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![id, self.game_id() as i64], |row| {
            Ok(DbGameStateSnapshot {
                id: row.get(0)?,
                game_id: self.game_id() as i64,
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

    fn list_games(&self) -> Result<Vec<Game>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, world_name, name, created_at, updated_at
                 FROM games
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare list games: {e}")))?;

        let db_games: Vec<DbGame> = stmt
            .query_map([], |row| {
                Ok(DbGame {
                    id: row.get(0)?,
                    world_name: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to list games: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| EngineError::Config(format!("Failed to read game row: {e}")))?;

        db_games.iter().map(db_game_to_game).collect()
    }

    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError> {
        let conn = self.pool.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            rusqlite::params![world_name, name, &now],
        )
        .map_err(|e| EngineError::Config(format!("Failed to create game: {e}")))?;
        Ok(conn.last_insert_rowid() as u64)
    }

    fn delete_game(&self, id: u64) -> Result<(), EngineError> {
        let mut conn = self.pool.conn();
        let tx = conn
            .transaction()
            .map_err(|e| EngineError::Config(format!("Failed to begin transaction: {e}")))?;
        tx.execute(
            "DELETE FROM game_state_snapshots WHERE game_id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete snapshots: {e}")))?;
        tx.execute(
            "DELETE FROM messages WHERE game_id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete messages: {e}")))?;
        tx.execute(
            "DELETE FROM games WHERE id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
        tx.commit()
            .map_err(|e| EngineError::Config(format!("Failed to commit transaction: {e}")))?;
        Ok(())
    }

    fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, world_name, name, created_at, updated_at
                 FROM games
                 WHERE id = ?1
                 LIMIT 1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare get game: {e}")))?;

        let db_result = stmt.query_row(rusqlite::params![id as i64], |row| {
            Ok(DbGame {
                id: row.get(0)?,
                world_name: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        });

        match db_result {
            Ok(db) => Ok(Some(db_game_to_game(&db)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EngineError::Config(format!("Failed to get game: {e}"))),
        }
    }
}
