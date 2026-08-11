//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings service — settings persistence orchestration at the application layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::domain::model::settings::AppSettings;
use crate::error::Result;

#[derive(Clone)]
pub struct SettingsService {
    storage: Arc<Storage>,
}

impl SettingsService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        self.storage.save_settings(settings)
    }
}
