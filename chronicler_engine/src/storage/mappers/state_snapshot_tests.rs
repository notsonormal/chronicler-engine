use std::collections::HashMap;

use chrono::Utc;

use crate::model::state::{MovementState, SceneState};
use crate::model::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::model::trigger::CharacterState;
use crate::storage::mappers::state_snapshot::snapshot_to_db;

#[test]
fn test_snapshot_roundtrip() {
    let original = GameStateSnapshot {
        db_id: Some(99),
        movement: MovementState {
            current_room_id: "room1".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState {
            npcs_in_area: vec![],
        },
        character_state: CharacterState::default(),
        committed: true,
        created_at: Utc::now(),
    };
    let db = snapshot_to_db(&original, 1).unwrap();
    let back = GameStateSnapshot::try_from(&db).unwrap();

    assert_eq!(original.db_id, back.db_id);
    assert_eq!(
        original.movement.current_room_id,
        back.movement.current_room_id
    );
    assert_eq!(original.committed, back.committed);
    assert_eq!(original.created_at, back.created_at);
    assert_eq!(db.game_id, 1);
}

#[test]
fn test_snapshot_uncommitted_no_db_id() {
    let original = GameStateSnapshot {
        db_id: None,
        movement: MovementState {
            current_room_id: "start".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState {
            npcs_in_area: vec![],
        },
        character_state: CharacterState::default(),
        committed: false,
        created_at: Utc::now(),
    };
    let db = snapshot_to_db(&original, 2).unwrap();

    assert_eq!(db.id, 0);
    assert_eq!(db.committed, 0);
    assert_eq!(db.game_id, 2);
}

#[test]
fn test_snapshot_json_columns() {
    let original = GameStateSnapshot {
        db_id: Some(1),
        movement: MovementState {
            current_room_id: "hallway".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState {
            npcs_in_area: vec![],
        },
        character_state: CharacterState::default(),
        committed: false,
        created_at: Utc::now(),
    };
    let db = snapshot_to_db(&original, 1).unwrap();

    assert!(db.movement_json.contains("hallway"));
    assert!(db.narrative_json.contains("generation"));
    assert!(db.scene_json.contains("npcs_in_area"));
    assert!(db.character_state_json.contains("npcs"));
}
