//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameCatalogue — game-lifecycle storage orchestration.

use std::sync::{Arc, RwLock};

use crate::application::errors::ApplicationError;
use crate::application::message_service::MessageService;
use crate::domain::model::game::Game;
use crate::domain::model::settings::{AppSettings, NarrativePerspective, NarrativeTense, NarratorMode};
use crate::domain::model::utils::game_name::generate_game_name;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::adapters::driven::storage::Storage;

#[derive(Clone)]
pub struct GameCatalogue {
    storage: Arc<Storage>,
    message_service: Arc<MessageService>,
    settings: Arc<RwLock<AppSettings>>,
}

impl GameCatalogue {
    pub fn new(
        storage: Arc<Storage>,
        message_service: Arc<MessageService>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        Self {
            storage,
            message_service,
            settings,
        }
    }

    pub fn create_game(&self, world_key: &str, persona_key: &str) -> Result<u64, ApplicationError> {
        let storage = &self.storage;
        let world_with_map = storage
            .get_world(world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let world_card = &world_with_map.world_card;
        let player = storage
            .get_persona(persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;
        let games = storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);

        let bundle = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            settings
                .mode_preset_registry
                .bundle_for(world_card.narrator_mode)
        };

        let request = crate::domain::model::game::NewGame {
            world_name: world_name.clone(),
            world_key: world_key.to_string(),
            persona_key: persona_key.to_string(),
            persona_name: player.sheet.name.clone(),
            name: name.clone(),
            narrator_mode: world_card.narrator_mode,
            narrative_perspective: world_card.narrative_perspective,
            narrative_tense: world_card.narrative_tense,
            system_prompt_preset_id: bundle.system_prompt_preset_id,
            quantifier_prompt_preset_id: bundle.quantifier_prompt_preset_id,
            impersonate_prompt_preset_id: bundle.impersonate_prompt_preset_id,
        };

        let new_id = storage.create_game_from_request(&request)?;
        let old_id = storage.current_game_id();
        self.storage.set_game_id(new_id);

        match self.persist_initial_state_with_swipes() {
            Ok(_) => {}
            Err(e) => {
                self.storage.set_game_id(old_id);
                return Err(e);
            }
        }

        Ok(new_id)
    }

    pub fn switch_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        self.storage.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, id: u64) -> Result<(), ApplicationError> {
        if id == self.storage.current_game_id() {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }
        self.storage.delete_game(id)?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<Game>, ApplicationError> {
        self.storage.list_games().map_err(Into::into)
    }

    pub fn current_game_id(&self) -> u64 {
        self.storage.current_game_id()
    }

    pub fn current_game(&self) -> Result<Option<Game>, ApplicationError> {
        let id = self.storage.current_game_id();
        self.storage.get_game(id).map_err(Into::into)
    }

    /// Override a game's perspective and tense (per-game posture override).
    /// Mode is deliberately absent — switching mode is `switch_mode`, which
    /// also retargets presets and nudges perspective.
    pub fn set_posture(
        &self,
        id: u64,
        perspective: NarrativePerspective,
        tense: NarrativeTense,
    ) -> Result<Game, ApplicationError> {
        let mut game = self.require_game(id)?;
        game.narrative_perspective = perspective;
        game.narrative_tense = tense;
        self.storage.update_game_config(&game)?;
        Ok(game)
    }

    /// Switch a game's narrator mode (the distinct mode-switch action):
    /// retarget the game's three preset-ids to the new mode's registry
    /// bundle, nudge perspective only when it still sits at the other mode's
    /// default, and persist. A deliberately-set perspective is never clobbered.
    pub fn switch_mode(&self, id: u64, mode: NarratorMode) -> Result<Game, ApplicationError> {
        let mut game = self.require_game(id)?;
        if game.narrator_mode == mode {
            return Ok(game);
        }

        let bundle = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            settings.mode_preset_registry.bundle_for(mode)
        };
        game.narrator_mode = mode;
        game.active_system_prompt_preset_id = bundle.system_prompt_preset_id;
        game.active_quantifier_prompt_preset_id = bundle.quantifier_prompt_preset_id;
        game.active_impersonate_prompt_preset_id = bundle.impersonate_prompt_preset_id;

        let other_default = match mode {
            NarratorMode::Novel => NarratorMode::InteractiveFiction,
            NarratorMode::InteractiveFiction => NarratorMode::Novel,
        }
        .default_perspective();
        if game.narrative_perspective == other_default {
            game.narrative_perspective = mode.default_perspective();
        }

        self.storage.update_game_config(&game)?;
        Ok(game)
    }

    /// Set the game's per-game preset selection (the picker). Each id must
    /// exist in the library; a mode-disallowed preset is accepted only when
    /// unchanged, so the stored selection is never silently invalidated.
    pub fn set_preset_selection(
        &self,
        id: u64,
        system_id: &str,
        quantifier_id: &str,
        impersonate_id: &str,
    ) -> Result<Game, ApplicationError> {
        let mut game = self.require_game(id)?;
        let selections = [
            (system_id, game.active_system_prompt_preset_id.clone()),
            (
                quantifier_id,
                game.active_quantifier_prompt_preset_id.clone(),
            ),
            (
                impersonate_id,
                game.active_impersonate_prompt_preset_id.clone(),
            ),
        ];
        for (new_id, current_id) in selections {
            // An unchanged stored id is a no-op slot: it skips validation so
            // saving the form never fails on a stale or deleted selection.
            if new_id == current_id {
                continue;
            }
            let preset = self.storage.get_preset(new_id)?.ok_or_else(|| {
                ApplicationError::validation(format!("Preset not found: {new_id}"))
            })?;
            if !preset.allows(game.narrator_mode) {
                return Err(ApplicationError::validation(format!(
                    "Preset not allowed for {} mode",
                    game.narrator_mode.as_str()
                )));
            }
        }
        game.active_system_prompt_preset_id = system_id.to_string();
        game.active_quantifier_prompt_preset_id = quantifier_id.to_string();
        game.active_impersonate_prompt_preset_id = impersonate_id.to_string();
        self.storage.update_game_config(&game)?;
        Ok(game)
    }

    fn require_game(&self, id: u64) -> Result<Game, ApplicationError> {
        self.storage
            .get_game(id)?
            .ok_or_else(|| ApplicationError::validation("Game not found"))
    }

    pub fn reset(&self) -> Result<(), ApplicationError> {
        let storage = &self.storage;
        let current_id = storage.current_game_id();
        let game = storage
            .get_game(current_id)?
            .ok_or_else(|| ApplicationError::validation("Current game not found"))?;
        let world_key = game.world_key.clone();
        let persona_key = game.persona_key.clone();

        let world_with_map = storage
            .get_world(&world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let world_card = &world_with_map.world_card;
        let player = storage
            .get_persona(&persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;

        storage.delete_game(current_id)?;

        let existing_names: Vec<String> = storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_key == world_key)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_name, &existing_names);
        let bundle = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            settings
                .mode_preset_registry
                .bundle_for(world_card.narrator_mode)
        };
        let request = crate::domain::model::game::NewGame {
            world_name: world_name.clone(),
            world_key: world_key.clone(),
            persona_key: persona_key.clone(),
            persona_name: player.sheet.name.clone(),
            name: new_name.clone(),
            narrator_mode: world_card.narrator_mode,
            narrative_perspective: world_card.narrative_perspective,
            narrative_tense: world_card.narrative_tense,
            system_prompt_preset_id: bundle.system_prompt_preset_id,
            quantifier_prompt_preset_id: bundle.quantifier_prompt_preset_id,
            impersonate_prompt_preset_id: bundle.impersonate_prompt_preset_id,
        };
        let new_id = storage.create_game_from_request(&request)?;
        self.storage.set_game_id(new_id);

        let _ = self.persist_initial_state_with_swipes();

        Ok(())
    }

    fn persist_initial_state_with_swipes(&self) -> Result<u64, ApplicationError> {
        let mut initial_state = self.message_service.build_fresh_initial_state()?;
        let storage = &self.storage;
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = storage.save_snapshot(&snapshot)?;

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                match storage.insert_message(&*msg) {
                    Ok(id) => {
                        msg.id = id;
                        storage.persist_swipes(id, &msg.swipes);
                    }
                    Err(e) => {
                        tracing::error!("persist_initial_state: message insert failed: {e}");
                    }
                }
            }
        }

        Ok(snapshot_id)
    }
}
