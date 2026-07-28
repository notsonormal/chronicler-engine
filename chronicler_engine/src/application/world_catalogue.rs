//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! WorldCatalogue — worlds/personas CRUD pass-through façade over `Storage`.
//! Takes `Arc<Storage>` directly (not `Arc<PersistenceGate>` like `GameCatalogue`):
//! Storage-direct CRUD only — no preset/snapshot/set_game_id needs. Narrowest collaborator.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::application::errors::ApplicationError;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::world::WorldCard;

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

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, ApplicationError> {
        self.storage.get_world(key).map_err(Into::into)
    }

    pub fn create_world(
        &self,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.storage
            .create_world(&world_card, &map)
            .map_err(Into::into)
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

    pub fn list_personas(&self) -> Result<Vec<PersonaCard>, ApplicationError> {
        self.storage.list_personas().map_err(Into::into)
    }
}
