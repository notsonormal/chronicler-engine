//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! GenerationSlot — per-game registry slot enum (distinct from domain `GenerationStatus`, which is the pipeline phase).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationSlot {
    /// No generation active for this game; slot is claimable.
    Idle,
    /// Generation `generation_id` owns the slot; releasing requires matching id.
    Generating { generation_id: u64 },
}

impl Default for GenerationSlot {
    fn default() -> Self {
        Self::Idle
    }
}

impl GenerationSlot {
    pub fn is_generating(&self) -> bool {
        matches!(self, Self::Generating { .. })
    }
}

/// Release a registry slot if the caller still owns it.
/// A caller that no longer owns its slot is a no-op.
pub fn release_owned_slot(
    registry: &Arc<RwLock<HashMap<u64, GenerationSlot>>>,
    game_id: u64,
    generation_id: u64,
) {
    let mut registry = registry.write().unwrap_or_else(|p| {
        tracing::warn!(
            "Generation registry write lock poisoned during release_owned_slot; recovering"
        );
        p.into_inner()
    });
    let still_owner = matches!(
        registry.get(&game_id),
        Some(GenerationSlot::Generating { generation_id: slot_gen }) if *slot_gen == generation_id
    );
    if still_owner {
        registry.insert(game_id, GenerationSlot::Idle);
    } else {
        tracing::debug!(
            game_id,
            generation_id,
            "release_owned_slot: registry slot not owned by caller; no-op"
        );
    }
}
