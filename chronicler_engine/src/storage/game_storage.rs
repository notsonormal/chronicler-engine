use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::EngineError;
use crate::model::game::Game;
use crate::storage::db::DbPool;
use crate::storage::models::game::DbGame;

pub trait GameStorage: Send + Sync {
    fn set_game_id(&self, game_id: u64);
    fn current_game_id(&self) -> u64;
    fn list_games(&self) -> Result<Vec<Game>, EngineError>;
    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError>;
    fn delete_game(&self, id: u64) -> Result<(), EngineError>;
    fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError>;
}

pub struct SqliteGameRepository {
    pool: DbPool,
    game_id: AtomicU64,
}

impl SqliteGameRepository {
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

fn parse_datetime(
    rfc3339: &str,
    field: &str,
) -> Result<chrono::DateTime<chrono::Utc>, EngineError> {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map_err(|e| EngineError::Config(format!("Invalid {field}: {e}")))
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn db_game_to_game(db: &DbGame) -> Result<Game, EngineError> {
    Ok(Game {
        id: db.id as u64,
        world_name: db.world_name.clone(),
        name: db.name.clone(),
        created_at: parse_datetime(&db.created_at, "created_at")?,
        updated_at: parse_datetime(&db.updated_at, "updated_at")?,
    })
}

impl GameStorage for SqliteGameRepository {
    fn set_game_id(&self, game_id: u64) {
        let current = self.game_id();
        if current != game_id {
            let conn = self.pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            if let Err(e) = conn.execute(
                "UPDATE games SET updated_at = ?1 WHERE id = ?2",
                rusqlite::params![&now, game_id as i64],
            ) {
                log::error!("Failed to update games.updated_at for game {game_id}: {e}");
            }
            self.game_id.store(game_id, Ordering::SeqCst);
        }
    }

    fn current_game_id(&self) -> u64 {
        self.game_id()
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
        let conn = self.pool.conn();
        conn.execute(
            "DELETE FROM games WHERE id = ?1",
            rusqlite::params![id as i64],
        )
        .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
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
