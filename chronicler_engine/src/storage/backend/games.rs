//! [DOC: docs/system/game_flow.md]
//! Game storage operations

use crate::error::EngineError;
use crate::model::game::Game;
use crate::storage::backend::{Backend, Storage};
use crate::storage::models::game::DbGame;

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
                    .query_map([], |row| {
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
                    })
                    .map_err(|e| EngineError::Config(format!("Failed to list games: {e}")))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| EngineError::Config(format!("Failed to read game row: {e}")))?;

                db_games.iter().map(db_game_to_game).collect()
            }
            Backend::InMemory(data) => {
                let mut games = data.games.clone();
                games.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                Ok(games)
            }
            Backend::Test { .. } => unreachable!(),
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
            Backend::Test { .. } => unreachable!(),
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
            Backend::Test { .. } => unreachable!(),
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
                    Ok(db) => Ok(Some(db_game_to_game(&db)?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!("Failed to get game: {e}"))),
                }
            }
            Backend::InMemory(data) => Ok(data.games.iter().find(|g| g.id == id).cloned()),
            Backend::Test { .. } => unreachable!(),
        })
    }
}

fn db_game_to_game(db: &DbGame) -> Result<Game, EngineError> {
    Ok(Game {
        id: db.id as u64,
        world_name: db.world_name.clone(),
        world_key: db.world_key.clone(), // NEW
        persona_key: db.persona_key.clone(),
        persona_name: db.persona_name.clone(),
        name: db.name.clone(),
        created_at: parse_datetime(&db.created_at, "created_at")?,
        updated_at: parse_datetime(&db.updated_at, "updated_at")?,
    })
}

fn parse_datetime(
    rfc3339: &str,
    field: &str,
) -> Result<chrono::DateTime<chrono::Utc>, EngineError> {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map_err(|e| EngineError::Config(format!("Invalid {field}: {e}")))
        .map(|dt| dt.with_timezone(&chrono::Utc))
}
