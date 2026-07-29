//! Tests for `slot.rs` `release_owned_slot` ownership race logic.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::application::utils::slot::{release_owned_slot, GenerationSlot};

fn registry_with(game_id: u64, slot: GenerationSlot) -> Arc<RwLock<HashMap<u64, GenerationSlot>>> {
    let mut map = HashMap::new();
    map.insert(game_id, slot);
    Arc::new(RwLock::new(map))
}

#[test]
fn release_owned_slot_clears_when_still_owner() {
    let game_id = 1;
    let registry = registry_with(game_id, GenerationSlot::Generating { generation_id: 100 });
    let is_generating = Arc::new(AtomicBool::new(true));

    release_owned_slot(&registry, &is_generating, game_id, 100);

    assert_eq!(
        registry.read().unwrap().get(&game_id),
        Some(&GenerationSlot::Idle)
    );
    assert!(!is_generating.load(Ordering::SeqCst));
}

#[test]
fn release_owned_slot_noop_when_stale_owner() {
    // Slot has been taken over by a younger generation (200); stale
    // caller (gen 100) must not clobber the live owner.
    let game_id = 1;
    let registry = registry_with(game_id, GenerationSlot::Generating { generation_id: 200 });
    let is_generating = Arc::new(AtomicBool::new(true));

    release_owned_slot(&registry, &is_generating, game_id, 100);

    assert_eq!(
        registry.read().unwrap().get(&game_id),
        Some(&GenerationSlot::Generating { generation_id: 200 })
    );
    assert!(is_generating.load(Ordering::SeqCst));
}

#[test]
fn release_owned_slot_keeps_projection_true_when_other_game_still_generating() {
    let mut map = HashMap::new();
    map.insert(1, GenerationSlot::Generating { generation_id: 100 });
    map.insert(2, GenerationSlot::Generating { generation_id: 200 });
    let registry = Arc::new(RwLock::new(map));
    let is_generating = Arc::new(AtomicBool::new(true));

    release_owned_slot(&registry, &is_generating, 1, 100);

    assert_eq!(
        registry.read().unwrap().get(&1),
        Some(&GenerationSlot::Idle)
    );
    // Game 2 still generating → projection stays true.
    assert!(is_generating.load(Ordering::SeqCst));
}
