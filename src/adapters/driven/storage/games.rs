//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game storage operations

use crate::error::{DatabaseContext, EngineError};
use crate::domain::model::game::{Game, GameId};
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::adapters::driven::storage::models::game::DbGame;

impl Storage {
    pub fn list_games(&self) -> Result<Vec<Game>, EngineError> {
        self.with_backend_mut(StorageCall::ListGames, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id, active_options_prompt_preset_id, options_always_on, display_name, failure
                         FROM games
                         ORDER BY updated_at DESC, id DESC",
                    )
                    .db_context("Failed to prepare list games")?;

                let db_games: Vec<DbGame> = stmt
                    .query_map([], DbGame::from_row)
                    .db_context("Failed to list games")?
                    .collect::<Result<Vec<_>, _>>()
                    .db_context("Failed to read game row")?;

                db_games.iter().map(DbGame::to_game).collect()
            }
        })
    }

    /// One page of the Games other than the active one, newest `updated_at` first. Equal
    /// `updated_at` lists the greater id first, so a page is stable.
    pub fn list_saved_games_page(
        &self,
        active: Option<GameId>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Game>, EngineError> {
        self.with_backend_mut(StorageCall::ListSavedGames, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id, active_options_prompt_preset_id, options_always_on, display_name, failure
                         FROM games
                         WHERE id IS NOT ?1
                         ORDER BY updated_at DESC, id DESC
                         LIMIT ?2 OFFSET ?3",
                    )
                    .db_context("Failed to prepare list saved games")?;

                let db_games: Vec<DbGame> = stmt
                    .query_map(
                        rusqlite::params![active.map(|id| id.0 as i64), limit as i64, offset as i64],
                        DbGame::from_row,
                    )
                    .db_context("Failed to list saved games")?
                    .collect::<Result<Vec<_>, _>>()
                    .db_context("Failed to read game row")?;

                db_games.iter().map(DbGame::to_game).collect()
            }
        })
    }

    pub fn count_saved_games(&self, active: Option<GameId>) -> Result<u64, EngineError> {
        self.with_backend_mut(StorageCall::CountSavedGames, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM games WHERE id IS NOT ?1",
                        rusqlite::params![active.map(|id| id.0 as i64)],
                        |row| row.get(0),
                    )
                    .db_context("Failed to count saved games")?;
                Ok(count as u64)
            }
        })
    }

    /// The uniqueness scan keys on `name`, so a rename must not touch it.
    pub fn update_game_display_name(&self, id: u64, display_name: &str) -> Result<(), EngineError> {
        self.with_backend_mut(
            StorageCall::UpdateGameDisplayName,
            |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let now = chrono::Utc::now().to_rfc3339();
                    let affected = conn
                        .execute(
                            "UPDATE games SET display_name=?, updated_at=? WHERE id=?",
                            rusqlite::params![display_name, &now, id as i64],
                        )
                        .db_context("Failed to update game display name")?;
                    if affected == 0 {
                        return Err(EngineError::GameNotFound(id));
                    }
                    Ok(())
                }
            },
        )
    }

    pub fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError> {
        self.with_backend_mut(StorageCall::GetGame, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id, active_options_prompt_preset_id, options_always_on, display_name, failure
                         FROM games
                         WHERE id = ?1
                         LIMIT 1",
                    )
                    .db_context("Failed to prepare get game")?;

                let db_result = stmt.query_row(rusqlite::params![id as i64], DbGame::from_row);

                match db_result {
                    Ok(db) => Ok(Some(db.to_game()?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e).db_context("Failed to get game"),
                }
            }
        })
    }

    pub fn require_game(&self, id: u64) -> Result<Game, EngineError> {
        self.get_game(id)?
            .ok_or_else(|| EngineError::GameNotFound(id))
    }

    /// The newest Game of a World: the one boot picks up. The order is the
    /// newest Message of the Game, else the Game's `updated_at`, which
    /// `ActiveGame::set` touches. The higher Game id breaks a tie in either
    /// key.
    pub fn newest_game_of_world(&self, world_key: &str) -> Result<Option<GameId>, EngineError> {
        self.with_backend_mut(StorageCall::NewestGameOfWorld, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT g.id
                         FROM games g
                         LEFT JOIN (
                             SELECT game_id, MAX(timestamp) AS last_message
                             FROM messages
                             GROUP BY game_id
                         ) m ON g.id = m.game_id
                         WHERE g.world_key = ?1
                         ORDER BY COALESCE(m.last_message, g.updated_at) DESC, g.id DESC
                         LIMIT 1",
                    )
                    .db_context("Failed to prepare newest game")?;
                let result = stmt.query_row(rusqlite::params![world_key], |row| {
                    Ok(GameId(row.get::<_, i64>(0)? as u64))
                });
                match result {
                    Ok(game_id) => Ok(Some(game_id)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e).db_context("Failed to read the newest game of the world"),
                }
            }
        })
    }

    pub fn count_games_using_preset(&self, preset_id: &str) -> Result<usize, EngineError> {
        self.with_backend_mut(
            StorageCall::CountGamesUsingPreset,
            |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let count: i64 = conn
                        .query_row(
                            "SELECT COUNT(*) FROM games
                         WHERE active_system_prompt_preset_id = ?1
                            OR active_quantifier_prompt_preset_id = ?1
                            OR active_impersonate_prompt_preset_id = ?1
                            OR active_options_prompt_preset_id = ?1",
                            [preset_id],
                            |row| row.get(0),
                        )
                        .db_context("Failed to count the games that use the preset")?;
                    Ok(count as usize)
                }
            },
        )
    }
}
