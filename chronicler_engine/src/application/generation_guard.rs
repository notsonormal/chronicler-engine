//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Generation guard logic

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;

use crate::application::generation_gate::slot::GenerationSlot;
use crate::application::generation_gate::slot::release_owned_slot;

/// RAII guard releasing the per-game registry slot on drop.
/// No-op if superseded by a younger generation (reset/switch_game).
pub struct GenerationGuard {
    game_id: u64,
    generation_id: u64,
    registry: Arc<RwLock<HashMap<u64, GenerationSlot>>>,
    is_generating: Arc<AtomicBool>,
}

impl GenerationGuard {
    pub fn new(
        game_id: u64,
        generation_id: u64,
        registry: Arc<RwLock<HashMap<u64, GenerationSlot>>>,
        is_generating: Arc<AtomicBool>,
    ) -> Self {
        Self {
            game_id,
            generation_id,
            registry,
            is_generating,
        }
    }
}

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        release_owned_slot(
            &self.registry,
            &self.is_generating,
            self.game_id,
            self.generation_id,
        );
    }
}
