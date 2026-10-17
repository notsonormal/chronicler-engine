//! Tests for `slot.rs` `release_owned_slot` ownership race logic.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::application::generation::slot::{release_owned_slot, GenerationSlot};
use crate::domain::model::game::GameId;

fn registry_with(
    game_id: GameId,
    slot: GenerationSlot,
) -> Arc<RwLock<HashMap<GameId, GenerationSlot>>> {
    let mut map = HashMap::new();
    map.insert(game_id, slot);
    Arc::new(RwLock::new(map))
}

#[test]
fn release_owned_slot_clears_when_still_owner() {
    let game_id = GameId(1);
    let registry = registry_with(game_id, GenerationSlot::Generating { generation_id: 100 });

    release_owned_slot(&registry, game_id, 100);

    assert_eq!(
        registry.read().unwrap().get(&game_id),
        Some(&GenerationSlot::Idle)
    );
}

#[test]
fn release_owned_slot_noop_when_stale_owner() {
    // Slot has been taken over by a younger generation (200); stale
    // caller (gen 100) must not clobber the live owner.
    let game_id = GameId(1);
    let registry = registry_with(game_id, GenerationSlot::Generating { generation_id: 200 });

    release_owned_slot(&registry, game_id, 100);

    assert_eq!(
        registry.read().unwrap().get(&game_id),
        Some(&GenerationSlot::Generating { generation_id: 200 })
    );
}

#[test]
fn release_owned_slot_only_affects_target_game() {
    let mut map = HashMap::new();
    map.insert(GameId(1), GenerationSlot::Generating { generation_id: 100 });
    map.insert(GameId(2), GenerationSlot::Generating { generation_id: 200 });
    let registry = Arc::new(RwLock::new(map));

    release_owned_slot(&registry, GameId(1), 100);

    assert_eq!(
        registry.read().unwrap().get(&GameId(1)),
        Some(&GenerationSlot::Idle)
    );
    // Game 2 still generating.
    assert_eq!(
        registry.read().unwrap().get(&GameId(2)),
        Some(&GenerationSlot::Generating { generation_id: 200 })
    );
}
