//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt preset service — prompt preset persistence orchestration at the application layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::error::Result;

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

    pub fn save_preset(&self, preset: &PromptPreset) -> Result<()> {
        self.storage.save_preset(preset)
    }

    pub fn delete_preset(&self, id: &str) -> Result<()> {
        self.storage.delete_preset(id)
    }
}
