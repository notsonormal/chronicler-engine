//! [DOC: docs/system/game_flow.md]
//! GenerationSlot — per-game registry slot enum (distinct from domain `GenerationStatus`, which is the pipeline phase).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationSlot {
    Idle,
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

/// Release a registry slot if the caller still owns it. Used by
/// `GenerationGuard::drop` and `GenerationGate::release_generation_slot` so
/// both paths share the same ownership check + projection atomic update.
///
/// A caller that no longer owns its slot (e.g. a stale generation whose game
/// was reset and the slot taken over) is a no-op — it must not clobber the
/// younger generation's slot or the projection atomic.
pub fn release_owned_slot(
    registry: &Arc<RwLock<HashMap<u64, GenerationSlot>>>,
    is_generating: &Arc<AtomicBool>,
    game_id: u64,
    generation_id: u64,
) {
    // Slot clear + projection store share the write lock so the two mutations
    // are atomic w.r.t. concurrent claims on other games.
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
    let any_other_generating = registry.values().any(|slot| slot.is_generating());
    if !any_other_generating {
        is_generating.store(false, Ordering::SeqCst);
    }
}
