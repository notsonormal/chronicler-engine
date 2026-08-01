//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! WorldPersonaCatalogue — world and persona storage orchestration.

use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::persistence_gate::PersistenceGate;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::world::WorldCard;

#[derive(Clone)]
pub struct WorldPersonaCatalogue {
    persistence_gate: Arc<PersistenceGate>,
}

impl WorldPersonaCatalogue {
    pub fn new(persistence_gate: Arc<PersistenceGate>) -> Self {
        Self { persistence_gate }
    }

    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, ApplicationError> {
        self.persistence_gate
            .storage()
            .list_worlds()
            .map_err(Into::into)
    }

    pub fn get_world(
        &self,
        key: &str,
    ) -> Result<Option<(i64, WorldCard, MapDef)>, ApplicationError> {
        self.persistence_gate
            .storage()
            .get_world(key)
            .map(|opt| opt.map(|w| (w.world_id, w.world_card, w.map)))
            .map_err(Into::into)
    }

    pub fn create_world(
        &self,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.persistence_gate
            .storage()
            .create_world(&world_card, &map)
            .map_err(Into::into)
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        self.persistence_gate
            .storage()
            .update_world(id, &world_card, &map)
            .map_err(Into::into)
    }

    pub fn delete_world(&self, key: &str) -> Result<(), ApplicationError> {
        self.persistence_gate
            .storage()
            .delete_world(key)
            .map_err(Into::into)
    }

    pub fn list_personas(&self) -> Result<Vec<PersonaCard>, ApplicationError> {
        self.persistence_gate
            .storage()
            .list_personas()
            .map_err(Into::into)
    }
}
