//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset service — prompt preset persistence orchestration at the application layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::domain::model::utils::name_is_available;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::error::{EngineError, Result};

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
            .map_err(Self::preset_error_to_application)
    }

    fn preset_error_to_application(error: EngineError) -> ApplicationError {
        match error {
            EngineError::Validation(message) => ApplicationError::Validation(message),
            other => ApplicationError::Engine(other),
        }
    }

    pub fn delete_preset(&self, id: &str) -> Result<()> {
        self.storage.delete_preset(id)
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
