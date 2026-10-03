//! [DOC: docs/diataxis/reference/game_flow.md]
//! World catalogue — world CRUD orchestration at the application layer.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::domain::model::map::MapDef;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

#[derive(Clone)]
pub struct WorldCatalogue {
    storage: Arc<Storage>,
}

impl WorldCatalogue {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, ApplicationError> {
        self.storage.list_worlds().map_err(Into::into)
    }

    pub fn get_world(
        &self,
        key: &str,
    ) -> Result<Option<(i64, WorldCard, MapDef)>, ApplicationError> {
        self.storage
            .get_world(key)
            .map(|opt| opt.map(|w| (w.world_id, w.world_card, w.map)))
            .map_err(Into::into)
    }

    /// Create a new world. Refuses a key that already exists — the user-facing
    /// create path must never silently replace an authored world. Bootstrap
    /// seeding uses the storage `seed_world` upsert instead.
    pub fn create_world(
        &self,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.storage
            .create_world(&world_card, &map)
            .map_err(|e| match e {
                EngineError::WorldAlreadyExists(_) => ApplicationError::validation(e.to_string()),
                other => ApplicationError::Engine(other),
            })
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        self.storage
            .update_world(id, &world_card, &map)
            .map_err(Into::into)
    }

    pub fn delete_world(&self, key: &str) -> Result<(), ApplicationError> {
        self.storage.delete_world(key).map_err(Into::into)
    }
}
