use std::sync::Arc;

use crate::model::state::GameState;
use crate::test_support::context::{make_test_context, make_test_context_with_sqlite};
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};

fn minimal_state() -> GameState {
    crate::model::state::GameStateBuilder::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        "start",
    )
    .build()
}

#[test]
fn test_make_test_context_roundtrip() {
    let state = minimal_state();
    let ctx = make_test_context(state);
    assert!(ctx.storage.load_latest_snapshot().unwrap().is_some());
    assert_eq!(ctx.world.name, "Test World");
}

#[test]
fn test_make_test_context_with_sqlite_roundtrip() {
    let state = minimal_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    assert!(ctx.storage.load_latest_snapshot().unwrap().is_some());
    assert_eq!(ctx.world.name, "Test World");
}
