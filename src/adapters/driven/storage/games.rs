//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game storage operations

use crate::error::EngineError;
use crate::domain::model::game::{Game, NewGame};
use crate::domain::model::settings::{
    AppSettings, ModePresetBundle, NarrativePerspective, NarrativeTense, NarratorMode,
};
use crate::adapters::driven::storage::{Backend, Storage};
use crate::adapters::driven::storage::models::game::DbGame;

impl Storage {
    pub fn list_games(&self) -> Result<Vec<Game>, EngineError> {
        self.with_backend_mut("list_games", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id
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
                    narrator_mode: NarratorMode::Novel,
                    narrative_perspective: NarrativePerspective::Third,
                    narrative_tense: NarrativeTense::Past,
                    active_system_prompt_preset_id: "system_default".to_string(),
                    active_quantifier_prompt_preset_id: "quantifier_default".to_string(),
                    active_impersonate_prompt_preset_id: "impersonate_default".to_string(),
                });
                Ok(id)
            }
        })
    }

    pub fn create_game_with_posture(&self, request: &NewGame) -> Result<u64, EngineError> {
        self.with_backend_mut("create_game", |backend| match backend {
            Backend::Sqlite { pool } => pool.insert_game_with_posture(request),
            Backend::InMemory(data) => {
                let id = data.next_game_id;
                data.next_game_id += 1;
                let now = chrono::Utc::now();
                data.games.push(Game {
                    id,
                    world_name: request.world_name.clone(),
                    world_key: request.world_key.clone(),
                    persona_key: request.persona_key.clone(),
                    persona_name: request.persona_name.clone(),
                    name: request.name.clone(),
                    created_at: now,
                    updated_at: now,
                    narrator_mode: request.narrator_mode,
                    narrative_perspective: request.narrative_perspective,
                    narrative_tense: request.narrative_tense,
                    active_system_prompt_preset_id: request.system_prompt_preset_id.clone(),
                    active_quantifier_prompt_preset_id: request.quantifier_prompt_preset_id.clone(),
                    active_impersonate_prompt_preset_id: request
                        .impersonate_prompt_preset_id
                        .clone(),
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
                        "SELECT id, world_name, name, created_at, updated_at, world_key, persona_key, persona_name, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id
                         FROM games
                         WHERE id = ?1
                         LIMIT 1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare get game: {e}")))?;

                let db_result = stmt.query_row(rusqlite::params![id as i64], DbGame::from_row);

                match db_result {
                    Ok(db) => Ok(Some(db.to_game()?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!("Failed to get game: {e}"))),
                }
            }
            Backend::InMemory(data) => Ok(data.games.iter().find(|g| g.id == id).cloned()),
        })
    }

    pub fn require_game(&self, id: u64) -> Result<Game, EngineError> {
        self.get_game(id)?
            .ok_or_else(|| EngineError::GameNotFound(id))
    }

    pub fn active_system_preset_id(&self, settings: &AppSettings) -> String {
        self.resolve_active_preset_id(
            settings,
            |g| g.active_system_prompt_preset_id.as_str(),
            |b| b.system_prompt_preset_id.as_str(),
        )
    }

    pub fn active_quantifier_preset_id(&self, settings: &AppSettings) -> String {
        self.resolve_active_preset_id(
            settings,
            |g| g.active_quantifier_prompt_preset_id.as_str(),
            |b| b.quantifier_prompt_preset_id.as_str(),
        )
    }

    pub fn active_impersonate_preset_id(&self, settings: &AppSettings) -> String {
        self.resolve_active_preset_id(
            settings,
            |g| g.active_impersonate_prompt_preset_id.as_str(),
            |b| b.impersonate_prompt_preset_id.as_str(),
        )
    }

    // Falls back to the registry's Novel bundle when the current game is absent.
    fn resolve_active_preset_id<G, B>(
        &self,
        settings: &AppSettings,
        game_slot: G,
        bundle_slot: B,
    ) -> String
    where
        G: FnOnce(&Game) -> &str,
        B: FnOnce(&ModePresetBundle) -> &str,
    {
        match self.get_game(self.current_game_id()).ok().flatten() {
            Some(game) => game_slot(&game).to_string(),
            None => {
                tracing::warn!("current game missing; falling back to registry novel preset");
                let bundle = settings
                    .mode_preset_registry
                    .bundle_for(NarratorMode::Novel);
                bundle_slot(&bundle).to_string()
            }
        }
    }
}
