use std::collections::HashMap;

use chrono::Utc;

use crate::model::state::{MovementState, SceneState};
use crate::model::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::model::trigger::NpcEncounterLog;
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
            ..Default::default()
        },
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: Utc::now(),
    };
    let db = snapshot_to_db(&original, 1).unwrap();
    let back = GameStateSnapshot::try_from(&db).unwrap();

    assert_eq!(original.db_id, back.db_id);
    assert_eq!(
        original.movement.current_room_id,
        back.movement.current_room_id
    );
    assert_eq!(original.created_at, back.created_at);
    assert_eq!(db.game_id, 1);
}

#[test]
fn test_snapshot_no_db_id_maps_correctly() {
    let original = GameStateSnapshot {
        db_id: None,
        movement: MovementState {
            current_room_id: "start".to_string(),
            dynamic_rooms: HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState {
            npcs_in_area: vec![],
            ..Default::default()
        },
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: Utc::now(),
    };
    let db = snapshot_to_db(&original, 2).unwrap();

    assert_eq!(db.id, 0);
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
            ..Default::default()
        },
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: Utc::now(),
    };
    let db = snapshot_to_db(&original, 1).unwrap();

    assert!(db.movement_json.contains("hallway"));
    assert!(db.narrative_json.contains("generation"));
    assert!(db.scene_json.contains("npcs_in_area"));
    assert!(db.npc_encounter_log_json.contains("npcs"));
}

#[test]
fn test_try_from_bad_movement_json() {
    let db = crate::storage::models::game_state_snapshot::DbGameStateSnapshot {
        id: 1,
        game_id: 1,
        movement_json: "not json".to_string(),
        narrative_json: "{}".to_string(),
        scene_json: "{}".to_string(),
        npc_encounter_log_json: "{}".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let result = GameStateSnapshot::try_from(&db);
    assert!(result.is_err());
}

#[test]
fn test_try_from_bad_narrative_json() {
    let valid_movement = serde_json::to_string(&MovementState {
        current_room_id: "room1".to_string(),
        dynamic_rooms: HashMap::new(),
    })
    .unwrap();
    let db = crate::storage::models::game_state_snapshot::DbGameStateSnapshot {
        id: 1,
        game_id: 1,
        movement_json: valid_movement,
        narrative_json: "not json".to_string(),
        scene_json: "{}".to_string(),
        npc_encounter_log_json: "{}".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let result = GameStateSnapshot::try_from(&db);
    assert!(result.is_err());
}

#[test]
fn test_try_from_bad_scene_json() {
    let valid_movement = serde_json::to_string(&MovementState {
        current_room_id: "room1".to_string(),
        dynamic_rooms: HashMap::new(),
    })
    .unwrap();
    let valid_narrative = serde_json::to_string(&NarrativeSnapshot::default()).unwrap();
    let db = crate::storage::models::game_state_snapshot::DbGameStateSnapshot {
        id: 1,
        game_id: 1,
        movement_json: valid_movement,
        narrative_json: valid_narrative,
        scene_json: "not json".to_string(),
        npc_encounter_log_json: "{}".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let result = GameStateSnapshot::try_from(&db);
    assert!(result.is_err());
}

#[test]
fn test_try_from_bad_npc_encounter_log_json() {
    let valid_movement = serde_json::to_string(&MovementState {
        current_room_id: "room1".to_string(),
        dynamic_rooms: HashMap::new(),
    })
    .unwrap();
    let valid_narrative = serde_json::to_string(&NarrativeSnapshot::default()).unwrap();
    let valid_scene = serde_json::to_string(&SceneState {
        npcs_in_area: vec![],
        ..Default::default()
    })
    .unwrap();
    let db = crate::storage::models::game_state_snapshot::DbGameStateSnapshot {
        id: 1,
        game_id: 1,
        movement_json: valid_movement,
        narrative_json: valid_narrative,
        scene_json: valid_scene,
        npc_encounter_log_json: "not json".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let result = GameStateSnapshot::try_from(&db);
    assert!(result.is_err());
}

#[test]
fn test_try_from_bad_created_at() {
    let valid_movement = serde_json::to_string(&MovementState {
        current_room_id: "room1".to_string(),
        dynamic_rooms: HashMap::new(),
    })
    .unwrap();
    let valid_narrative = serde_json::to_string(&NarrativeSnapshot::default()).unwrap();
    let valid_scene = serde_json::to_string(&SceneState {
        npcs_in_area: vec![],
        ..Default::default()
    })
    .unwrap();
    let valid_npc_log = serde_json::to_string(&NpcEncounterLog::default()).unwrap();
    let db = crate::storage::models::game_state_snapshot::DbGameStateSnapshot {
        id: 1,
        game_id: 1,
        movement_json: valid_movement,
        narrative_json: valid_narrative,
        scene_json: valid_scene,
        npc_encounter_log_json: valid_npc_log,
        created_at: "not-a-date".to_string(),
    };
    let result = GameStateSnapshot::try_from(&db);
    assert!(result.is_err());
}
