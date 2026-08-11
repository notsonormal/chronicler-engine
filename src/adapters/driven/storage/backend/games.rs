//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game storage operations

use crate::error::EngineError;
use crate::domain::model::game::Game;
use crate::adapters::driven::storage::backend::{Backend, Storage};
use crate::adapters::driven::storage::models::game::DbGame;
use crate::adapters::driven::storage::utils::parse_datetime;

impl Storage {
    pub fn list_games(&self) -> Result<Vec<Game>, EngineError> {
        self.with_backend_mut("list_games", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name
                         FROM games
                         ORDER BY updated_at DESC",
                    )
                    .map_err(|e| {
                        EngineError::Config(format!("Failed to prepare list games: {e}"))
                    })?;

                let db_games: Vec<DbGame> = stmt
                    .query_map([], DbGame::from_row)
                    .map_err(|e| EngineError::Config(format!("Failed to list games: {e}")))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| EngineError::Config(format!("Failed to read game row: {e}")))?;

                db_games.iter().map(DbGame::to_game).collect()
            }
            Backend::InMemory(data) => {
                let mut games = data.games.clone();
                games.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                Ok(games)
            }
        })
    }

    pub fn create_game(
        &self,
        world_name: &str,
        world_key: &str,
        persona_key: &str,
        persona_name: &str,
        name: &str,
    ) -> Result<u64, EngineError> {
        self.with_backend_mut("create_game", |backend| match backend {
            Backend::Sqlite { pool } => {
                pool.insert_game(world_name, world_key, persona_key, persona_name, name)
            }
            Backend::InMemory(data) => {
                let id = data.next_game_id;
                data.next_game_id += 1;
                let now = chrono::Utc::now();
                data.games.push(Game {
                    id,
                    world_name: world_name.to_string(),
                    world_key: world_key.to_string(),
                    persona_key: persona_key.to_string(),
                    persona_name: persona_name.to_string(),
                    name: name.to_string(),
                    created_at: now,
                    updated_at: now,
                });
                Ok(id)
            }
        })
    }

    pub fn delete_game(&self, id: u64) -> Result<(), EngineError> {
        self.with_backend_mut("delete_game", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "DELETE FROM games WHERE id = ?1",
                    rusqlite::params![id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.games.retain(|g| g.id != id);
                Ok(())
            }
        })
    }

    pub fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError> {
        self.with_backend_mut("get_game", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name
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
                        world_key: row.get(5)?,
                        persona_key: row.get(6)?,
                        persona_name: row.get(7)?,
                    })
                });

                match db_result {
                    Ok(db) => Ok(Some(db.to_game()?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!("Failed to get game: {e}"))),
                }
            }
            Backend::InMemory(data) => Ok(data.games.iter().find(|g| g.id == id).cloned()),
        })
    }

    /// Required-read of a game row by id. Absence becomes
    /// [`EngineError::GameNotFound`]. Optional / fallback / existence /
    /// validation callers should stay on [`Storage::get_game`](Self::get_game).
    pub fn require_game(&self, id: u64) -> Result<Game, EngineError> {
        self.get_game(id)?
            .ok_or_else(|| EngineError::GameNotFound(id))
    }
}

impl DbGame {
    fn to_game(&self) -> Result<Game, EngineError> {
        Ok(Game {
            id: self.id as u64,
            world_name: self.world_name.clone(),
            world_key: self.world_key.clone(),
            persona_key: self.persona_key.clone(),
            persona_name: self.persona_name.clone(),
            name: self.name.clone(),
            created_at: parse_datetime(&self.created_at, "created_at")?,
            updated_at: parse_datetime(&self.updated_at, "updated_at")?,
        })
    }
}
