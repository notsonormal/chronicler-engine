//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Persona catalogue — persona read-side orchestration at the application layer.
//!
//! This catalogue is intentionally a one-method seam. It keeps the HTTP layer from
//! calling `Storage` directly for persona lookups, mirroring the storage-isolation
//! discipline enforced for all other storage access in the HTTP layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::domain::model::character::PersonaCard;

#[derive(Clone)]
pub struct PersonaCatalogue {
    storage: Arc<Storage>,
}

impl PersonaCatalogue {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn list_personas(&self) -> Result<Vec<PersonaCard>, ApplicationError> {
        self.storage.list_personas().map_err(Into::into)
    }
}
