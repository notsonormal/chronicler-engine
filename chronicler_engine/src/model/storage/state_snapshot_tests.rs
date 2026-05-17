use std::collections::HashMap;
use std::sync::Arc;

use crate::model::state::{GameState, MovementState, NarrativeState, SceneState};
use crate::model::storage::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::model::trigger::CharacterState;
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};

#[test]
fn test_narrative_snapshot_default() {
    let snap = NarrativeSnapshot::default();
    assert_eq!(
        snap.generation.status,
        crate::model::state::GenerationStatus::Idle
    );
    assert!(snap.last_trigger.is_none());
    assert!(snap.pending_location.is_none());
    assert!(snap.pending_event.is_none());
}

#[test]
fn test_game_state_snapshot_apply_to() {
    let mut state = GameState {
        world: Arc::new(TestWorld::minimal()),
        map: Arc::new(TestMap::single_room("old_room")),
        player: Arc::new(TestPlayer::named("Test")),
        npcs: HashMap::new(),
        movement: MovementState {
            current_room_id: "old_room".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: NarrativeState::default(),
        scene: SceneState {
            npcs_in_area: vec![],
        },
        character_state: CharacterState::default(),
    };

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
        },
        character_state: CharacterState::default(),
        committed: true,
        created_at: chrono::Utc::now(),
    };

    snapshot.apply_to(&mut state);
    assert_eq!(state.movement.current_room_id, "new_room");
    assert_eq!(state.narrative.pending_location, Some("loc".to_string()));
    assert_eq!(state.narrative.pending_event, Some("evt".to_string()));
}
