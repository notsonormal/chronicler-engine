use std::collections::HashMap;
use std::sync::Arc;

use crate::model::state::{GameStateBuilder, MovementState, SceneState};
use crate::model::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::model::trigger::NpcEncounterLog;
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};

#[test]
fn test_narrative_snapshot_default() {
    let snap = NarrativeSnapshot::default();
    assert_eq!(
        snap.input_buffer.status,
        crate::model::state::GenerationStatus::Idle
    );
    assert!(snap.last_trigger.is_none());
    assert!(snap.pending_location.is_none());
    assert!(snap.pending_event.is_none());
}

#[test]
fn test_game_state_snapshot_apply_to() {
    let mut state = GameStateBuilder::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("old_room")),
        Arc::new(TestPlayer::named("Test")),
        "old_room",
    )
    .build();

    let snapshot = GameStateSnapshot {
        db_id: None,
        movement: MovementState {
            current_room_id: "new_room".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: NarrativeSnapshot {
            pending_location: Some("loc".to_string()),
            pending_event: Some("evt".to_string()),
            ..Default::default()
        },
        scene: SceneState {
            npcs_in_area: vec![],
            ..Default::default()
        },
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: chrono::Utc::now(),
    };

    snapshot.apply_to(&mut state);
    assert_eq!(state.movement.current_room_id, "new_room");
    assert_eq!(state.narrative.pending_location, Some("loc".to_string()));
    assert_eq!(state.narrative.pending_event, Some("evt".to_string()));
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
