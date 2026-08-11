//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Generation guard logic

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::application::generation::slot::GenerationSlot;
use crate::application::generation::slot::release_owned_slot;

/// RAII guard releasing the per-game registry slot on drop.
/// No-op if superseded by a younger generation (reset/switch_game).
pub struct GenerationGuard {
    game_id: u64,
    generation_id: u64,
    registry: Arc<RwLock<HashMap<u64, GenerationSlot>>>,
}

impl GenerationGuard {
    pub fn new(
        game_id: u64,
        generation_id: u64,
        registry: Arc<RwLock<HashMap<u64, GenerationSlot>>>,
    ) -> Self {
        Self {
            game_id,
            generation_id,
            registry,
        }
    }
}

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        release_owned_slot(&self.registry, self.game_id, self.generation_id);
    }
}
