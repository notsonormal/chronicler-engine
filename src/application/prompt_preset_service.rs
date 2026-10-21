//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt preset service — prompt preset persistence orchestration at the application layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::error::Result;
use crate::domain::model::game::Game;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::utils::name_is_available;
use crate::error::EngineError;

/// The fields of a preset an edit sets. The type, the id and the default flag
/// stay as stored.
#[derive(Debug, Clone)]
pub struct PresetEdit {
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    /// `None` keeps the stored modes.
    pub allowed_modes: Option<Vec<NarratorMode>>,
}

#[derive(Clone)]
pub struct PromptPresetService {
    storage: Arc<Storage>,
}

impl PromptPresetService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn get_preset(&self, id: &str) -> Result<Option<PromptPreset>> {
        self.storage.get_preset(id)
    }

    pub fn list_presets(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>> {
        self.storage.list_presets(preset_type)
    }

    fn name_available(siblings: &[PromptPreset], candidate: &str, except_id: &str) -> bool {
        name_is_available(
            siblings
                .iter()
                .map(|sibling| (sibling.id.as_str(), sibling.name.as_str())),
            candidate,
            Some(except_id),
        )
    }

    pub fn save_preset(&self, preset: &PromptPreset) -> std::result::Result<(), ApplicationError> {
        self.storage
            .save_preset(preset)
            .map_err(ApplicationError::from_engine_error)
    }

    /// The preset a Game picks for the Narrator. A missing preset is an error.
    pub fn for_narration(&self, game: &Game, preset_type: PresetType) -> Result<PromptPreset> {
        self.chosen(game, preset_type)
    }

    fn chosen(&self, game: &Game, preset_type: PresetType) -> Result<PromptPreset> {
        let id = game.active_preset_id(preset_type);
        self.storage
            .get_preset(id)?
            .ok_or_else(|| EngineError::PresetNotFound(id.to_string()))
    }

    /// The assembled text of the preset a Game picks for a hook. A missing preset
    /// is an error.
    pub fn hook_text(&self, game: &Game, preset_type: PresetType) -> Result<String> {
        Ok(self
            .chosen(game, preset_type)?
            .assemble_text(&[], None, None))
    }

    /// The stored preset an edit may change: one that exists and is not a default.
    pub fn editable(&self, id: &str) -> std::result::Result<PromptPreset, ApplicationError> {
        let preset = self
            .get_preset(id)?
            .ok_or_else(|| ApplicationError::validation("Preset not found"))?;
        if preset.is_default {
            return Err(ApplicationError::validation("Cannot edit default presets"));
        }
        Ok(preset)
    }

    pub fn edit_preset(
        &self,
        id: &str,
        edit: PresetEdit,
    ) -> std::result::Result<PromptPreset, ApplicationError> {
        let mut preset = self.editable(id)?;
        preset.name = edit.name;
        preset.role = edit.role;
        preset.instructions = edit.instructions;
        preset.writing_style = edit.writing_style;
        preset.output_format = edit.output_format;
        if let Some(modes) = edit.allowed_modes {
            preset.allowed_modes = modes;
        }
        self.save_preset(&preset)?;
        Ok(preset)
    }

    /// A delete is refused for a default preset, for a preset that a mode or the
    /// Options setting names as its default, and for a preset that a Game uses.
    pub fn delete_preset(&self, id: &str) -> std::result::Result<(), ApplicationError> {
        let preset = self
            .get_preset(id)?
            .ok_or_else(|| ApplicationError::validation("Preset not found"))?;
        if preset.is_default {
            return Err(ApplicationError::validation(
                "Cannot delete default presets",
            ));
        }
        let settings = self.storage.get_settings()?;
        if settings.mode_preset_registry.references(id) {
            return Err(ApplicationError::validation(
                "Preset is a mode default; change the default before deleting",
            ));
        }
        if settings.active_preset_id(PresetType::Options, NarratorMode::Novel) == id {
            return Err(ApplicationError::validation(
                "Preset is the default Options preset; change the default before deleting",
            ));
        }
        let game_count = self.storage.count_games_using_preset(id)?;
        if game_count > 0 {
            return Err(ApplicationError::validation(format!(
                "Cannot delete preset used by {}",
                EngineError::count_of_games(game_count)
            )));
        }
        self.storage.delete_preset(id)?;
        Ok(())
    }

    pub fn next_copy_name(
        &self,
        source: &PromptPreset,
    ) -> std::result::Result<String, ApplicationError> {
        let siblings = self.storage.list_presets(source.preset_type)?;
        let available = |candidate: &str| Self::name_available(&siblings, candidate, &source.id);
        let base = format!("{} (Copy)", source.name);
        if available(&base) {
            return Ok(base);
        }
        let mut copy_number = 2;
        loop {
            let candidate = format!("{} (Copy {copy_number})", source.name);
            if available(&candidate) {
                return Ok(candidate);
            }
            copy_number += 1;
        }
    }
}
