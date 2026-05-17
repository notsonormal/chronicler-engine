use std::collections::HashMap;
use std::sync::Arc;

use crate::model::state::GameState;
use crate::test_support::context::{make_test_context, make_test_context_with_sqlite};
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};

fn minimal_state() -> GameState {
    GameState {
        world: Arc::new(TestWorld::minimal()),
        map: Arc::new(TestMap::single_room("start")),
        player: Arc::new(TestPlayer::named("Test")),
        npcs: HashMap::new(),
        movement: crate::model::state::MovementState {
            current_room_id: "start".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: crate::model::state::NarrativeState::default(),
        scene: crate::model::state::SceneState {
            npcs_in_area: vec![],
        },
        npc_encounter_log: crate::model::trigger::NpcEncounterLog::default(),
    }
}

#[test]
fn test_make_test_context_roundtrip() {
    let state = minimal_state();
    let ctx = make_test_context(state);
    assert!(ctx.snapshot_storage.load_latest().unwrap().is_some());
    assert_eq!(ctx.world.name, "Test World");
}

#[test]
fn test_make_test_context_with_sqlite_roundtrip() {
    let state = minimal_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    assert!(ctx.snapshot_storage.load_latest().unwrap().is_some());
    assert_eq!(ctx.world.name, "Test World");
}
