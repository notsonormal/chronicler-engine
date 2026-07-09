//! [DOC: docs/system/game_flow.md]
//! WorldSnapshot DTO — persistence load bundle for an active game
//! (T2 ticket 02 — moved from DefaultApplicationService).

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::character::{NpcCard, PlayerCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::world::WorldCard;

#[derive(Clone)]
pub struct WorldSnapshot {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
}

impl WorldSnapshot {
    pub fn empty() -> Self {
        Self {
            world: Arc::new(WorldCard::default()),
            map: Arc::new(MapDef::default()),
            player: Arc::new(PlayerCard::default()),
            npcs: Arc::new(HashMap::new()),
        }
    }
}
