use std::sync::Arc;

use crate::domain::model::state::game_state::GameStateBuilder;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::adapters::driven::storage::snapshot_blob::{GameStateSnapshot, NarrativeSnapshot};
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};

#[test]
fn test_narrative_snapshot_default() {
    let snap = NarrativeSnapshot::default();
    assert_eq!(snap.input_buffer.status, GenerationStatus::Idle);
    assert!(snap.last_trigger.is_none());
    assert!(snap.pending_location.is_none());
    assert!(snap.pending_event.is_none());
}

#[test]
fn test_from_game_state_sets_defaults() {
    let state = GameStateBuilder::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        "start",
    )
    .build();

    let snapshot = GameStateSnapshot::from_game_state(&state);
    assert!(snapshot.db_id.is_none());
    assert!(
        snapshot.created_at <= chrono::Utc::now(),
        "created_at should be in the past"
    );
}
