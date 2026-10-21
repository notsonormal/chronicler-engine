//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game storage operations

use crate::error::{DatabaseContext, EngineError};
use crate::domain::model::game::{Game, GameId};
use crate::domain::model::settings::{AppSettings, NarratorMode};
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
                         ORDER BY updated_at DESC",
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

    pub fn update_game_config(&self, game: &Game) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::UpdateGameConfig, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                let affected = conn
                    .execute(
                        "UPDATE games SET narrator_mode=?, narrative_perspective=?, narrative_tense=?, active_system_prompt_preset_id=?, active_quantifier_prompt_preset_id=?, active_impersonate_prompt_preset_id=?, active_options_prompt_preset_id=?, options_always_on=?, updated_at=? WHERE id=?",
                        rusqlite::params![
                            game.narrator_mode.as_str(),
                            game.narrative_perspective.as_str(),
                            game.narrative_tense.as_str(),
                            game.active_system_prompt_preset_id,
                            game.active_quantifier_prompt_preset_id,
                            game.active_impersonate_prompt_preset_id,
                            game.active_options_prompt_preset_id,
                            game.options_always_on as i64,
                            &now,
                            game.id as i64,
                        ],
                    )
                    .db_context("Failed to update game config")?;
                if affected == 0 {
                    return Err(EngineError::GameNotFound(game.id));
                }
                Ok(())
            }
        })
    }

    pub fn active_quantifier_preset_id(&self, game_id: GameId, settings: &AppSettings) -> String {
        self.preset_id_or(
            game_id,
            || {
                tracing::warn!("game {game_id} missing; falling back to registry novel preset");
                settings
                    .mode_preset_registry
                    .bundle_for(NarratorMode::Novel)
                    .quantifier_prompt_preset_id
            },
            |game| &game.active_quantifier_prompt_preset_id,
        )
    }

    /// Options presets are mode-agnostic and sit outside the per-mode registry, so
    /// the fallback is the settings column, not a mode bundle.
    pub fn active_options_preset_id(&self, game_id: GameId, settings: &AppSettings) -> String {
        self.preset_id_or(
            game_id,
            || {
                tracing::warn!("game {game_id} missing; falling back to settings options preset");
                settings.active_options_prompt_preset_id.clone()
            },
            |game| &game.active_options_prompt_preset_id,
        )
    }

    /// The Game's preset slot, or `fallback` when the Game row is gone.
    fn preset_id_or(
        &self,
        game_id: GameId,
        fallback: impl FnOnce() -> String,
        game_slot: impl FnOnce(&Game) -> &str,
    ) -> String {
        match self.get_game(game_id.0).ok().flatten() {
            Some(game) => game_slot(&game).to_string(),
            None => fallback(),
        }
    }
}
