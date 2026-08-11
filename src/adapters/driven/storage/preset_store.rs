//! [DOC: docs/diataxis/reference/storage.md]
//! PresetStore newtype — distinguishes preset storage from game storage

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::error::EngineError;

#[derive(Clone)]
pub struct PresetStore {
    storage: Arc<Storage>,
}

impl PresetStore {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn inner(&self) -> &Arc<Storage> {
        &self.storage
    }

    pub fn get_preset(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        self.storage.get_preset(id)
    }

    pub fn save_preset(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        self.storage.save_preset(preset)
    }

    pub fn delete_preset(&self, id: &str) -> Result<(), EngineError> {
        self.storage.delete_preset(id)
    }

    pub fn list_presets(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        self.storage.list_presets(preset_type)
    }
}
