//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameCatalogue — game-lifecycle storage orchestration.

use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::error::EngineError;
use crate::application::games::active::ActiveGame;
use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::game::{ConfigChange, Game, GameId, NewGame, PresetSelection};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::domain::model::utils::game_name::generate_game_name;
use crate::adapters::driven::storage::Storage;
use crate::application::generation::gate::GameWriter;

#[derive(Clone)]
pub struct GameCatalogue {
    storage: Arc<Storage>,
    game_writer: GameWriter,
    active_game: Arc<ActiveGame>,
}

impl GameCatalogue {
    pub fn new(
        storage: Arc<Storage>,
        game_writer: GameWriter,
        active_game: Arc<ActiveGame>,
        initial_game: Option<GameId>,
    ) -> Result<Self, EngineError> {
        if let Some(initial_game) = initial_game {
            active_game.set(initial_game)?;
        }
        Ok(Self {
            storage,
            game_writer,
            active_game,
        })
    }

    pub fn create_game(&self, world_key: &str, persona_key: &str) -> Result<u64, ApplicationError> {
        let storage = &self.storage;
        let world_with_map = storage
            .get_world(world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let player = storage
            .get_persona(persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;
        let existing_names: Vec<String> = storage
            .list_games()?
            .iter()
            .map(|g| g.name.clone())
            .collect();
        let name = generate_game_name(&world_with_map.world_card.name, &existing_names);
        let request =
            self.new_game_request(&world_with_map, &player, world_key, persona_key, name)?;

        // One create unit: the Game, its empty dock record and the opening Narration.
        let new_id = self.game_writer.create(&request)?;
        // The pointer moves only once the unit landed, so a failed create leaves the
        // previous Game active.
        self.active_game.set_new(new_id);

        Ok(new_id.0)
    }

    pub fn switch_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        self.active_game.set(GameId(id))?;
        Ok(())
    }

    /// Display names may collide; the generated `name` stays the unique key.
    pub fn rename_game(&self, id: u64, display_name: &str) -> Result<Game, ApplicationError> {
        let display_name = display_name.trim();
        if display_name.is_empty() {
            return Err(ApplicationError::validation("Display name cannot be empty"));
        }
        let mut game = self.require_game(id)?;
        self.storage.update_game_display_name(id, display_name)?;
        game.display_name = display_name.to_string();
        Ok(game)
    }

    pub fn delete_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.active_game.get() == Some(GameId(id)) {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }
        self.game_writer.delete(GameId(id))?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<Game>, ApplicationError> {
        self.storage.list_games().map_err(Into::into)
    }

    pub fn newest_game_of_world(
        &self,
        world_key: &str,
    ) -> Result<Option<GameId>, ApplicationError> {
        self.storage
            .newest_game_of_world(world_key)
            .map_err(Into::into)
    }

    pub fn active_game_id(&self) -> Option<GameId> {
        self.active_game.get()
    }

    pub fn active_game(&self) -> Result<Option<Game>, ApplicationError> {
        let Some(id) = self.active_game.get() else {
            return Ok(None);
        };
        self.storage.get_game(id.0).map_err(Into::into)
    }

    /// Only perspective and tense: a mode switch is `switch_mode`, which also
    /// retargets presets and nudges perspective.
    pub fn set_posture(
        &self,
        id: u64,
        perspective: NarrativePerspective,
        tense: NarrativeTense,
    ) -> Result<Game, ApplicationError> {
        let game = self.require_game(id)?;
        self.save_config(game, ConfigChange::Posture { perspective, tense })
    }

    /// A deliberately-set perspective survives the switch; only the other mode's
    /// default is nudged.
    pub fn switch_mode(&self, id: u64, mode: NarratorMode) -> Result<Game, ApplicationError> {
        let game = self.require_game(id)?;
        if game.narrator_mode == mode {
            return Ok(game);
        }

        let bundle = {
            let settings = self.storage.get_settings()?;
            settings.mode_preset_registry.bundle_for(mode)
        };

        let other_default = match mode {
            NarratorMode::Novel => NarratorMode::InteractiveFiction,
            NarratorMode::InteractiveFiction => NarratorMode::Novel,
        }
        .default_perspective();
        let perspective = if game.narrative_perspective == other_default {
            mode.default_perspective()
        } else {
            game.narrative_perspective
        };

        self.save_config(
            game,
            ConfigChange::Mode {
                mode,
                system_id: bundle.system_prompt_preset_id,
                quantifier_id: bundle.quantifier_prompt_preset_id,
                impersonate_id: bundle.impersonate_prompt_preset_id,
                perspective,
            },
        )
    }

    /// A mode-tagged preset that disallows the game's mode is accepted only when
    /// unchanged, so a stored selection is never silently invalidated.
    pub fn set_preset_selection(
        &self,
        id: u64,
        selection: PresetSelection,
    ) -> Result<Game, ApplicationError> {
        let game = self.require_game(id)?;
        for (preset_type, new_id) in selection.slots() {
            // An unchanged stored id skips validation, so saving the form never
            // fails on a stale or deleted selection.
            if new_id == game.active_preset_id(preset_type) {
                continue;
            }
            let preset = self.storage.get_preset(new_id)?.ok_or_else(|| {
                ApplicationError::validation(format!("Preset not found: {new_id}"))
            })?;
            // Options presets serve both modes from one slot, so their
            // allowed_modes never gate the selection.
            if preset_type.is_mode_tagged() && !preset.allows(game.narrator_mode) {
                return Err(ApplicationError::validation(format!(
                    "Preset not allowed for {} mode",
                    game.narrator_mode.as_str()
                )));
            }
        }
        self.save_config(game, ConfigChange::Presets(selection))
    }

    /// The config rules ran outside the writer's lock; the write goes through the
    /// writer, which refuses it while an Autoplay run holds the Game. The write
    /// holds only the columns of `change`, and the returned Game is `game` with
    /// the same change applied.
    fn save_config(&self, mut game: Game, change: ConfigChange) -> Result<Game, ApplicationError> {
        self.game_writer.save_config(GameId(game.id), &change)?;
        change.apply_to(&mut game);
        Ok(game)
    }

    fn require_game(&self, id: u64) -> Result<Game, ApplicationError> {
        self.storage
            .get_game(id)?
            .ok_or_else(|| ApplicationError::validation("Game not found"))
    }

    pub fn reset(&self, current_id: GameId) -> Result<(), ApplicationError> {
        let storage = &self.storage;
        let game = storage
            .get_game(current_id.0)?
            .ok_or_else(|| ApplicationError::validation("Game not found"))?;
        let world_key = game.world_key.clone();
        let persona_key = game.persona_key.clone();

        let world_with_map = storage
            .get_world(&world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let player = storage
            .get_persona(&persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;

        // The Game being replaced is not yet gone when the name is picked, and the
        // fresh Game takes its place: its name is no more taken than a removed
        // Game's is.
        let existing_names: Vec<String> = storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_key == world_key && g.id != current_id.0)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_with_map.world_card.name, &existing_names);
        let request =
            self.new_game_request(&world_with_map, &player, &world_key, &persona_key, new_name)?;
        // The reset creates the fresh Game first and then removes the old one, in
        // one hold of the writer's guard, so a refused create leaves the old Game
        // and the pointer as they were.
        let new_id = self.game_writer.replace(current_id, &request)?;
        self.active_game.set_new(new_id);
        Ok(())
    }

    /// The create request both `create_game` and `reset` write. The caller picks the
    /// name, because the two name scans differ.
    fn new_game_request(
        &self,
        world: &WorldWithMap,
        persona: &PersonaCard,
        world_key: &str,
        persona_key: &str,
        name: String,
    ) -> Result<NewGame, ApplicationError> {
        let world_card = &world.world_card;
        let settings = self.storage.get_settings()?;
        let bundle = settings
            .mode_preset_registry
            .bundle_for(world_card.narrator_mode);
        Ok(NewGame {
            world_name: world_card.name.clone(),
            world_key: world_key.to_string(),
            persona_key: persona_key.to_string(),
            persona_name: persona.sheet.name.clone(),
            name,
            narrator_mode: world_card.narrator_mode,
            narrative_perspective: world_card.narrative_perspective,
            narrative_tense: world_card.narrative_tense,
            system_prompt_preset_id: bundle.system_prompt_preset_id,
            quantifier_prompt_preset_id: bundle.quantifier_prompt_preset_id,
            impersonate_prompt_preset_id: bundle.impersonate_prompt_preset_id,
            options_prompt_preset_id: settings.active_options_prompt_preset_id.clone(),
            options_always_on: world_card.options_always_on,
        })
    }
}
