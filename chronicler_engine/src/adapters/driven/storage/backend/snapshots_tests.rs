use crate::domain::model::state::movement::MovementState;
use crate::domain::model::state::scene_state::SceneState;
use crate::domain::model::state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::domain::model::trigger::NpcEncounterLog;
use crate::adapters::driven::storage::backend::{Storage, TestOverride};
use crate::test_support::sqlite_storage;

fn dummy_snapshot() -> GameStateSnapshot {
    GameStateSnapshot {
        db_id: None,
        movement: MovementState {
            current_room_id: "start".to_string(),
            dynamic_rooms: std::collections::HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState::default(),
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: chrono::Utc::now(),
    }
}

#[test]
fn test_save_snapshot_returns_positive_id() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();
    assert!(id > 0);
}

#[test]
fn test_save_snapshot_sqlite() {
    let storage = sqlite_storage().unwrap();
    storage.set_game_id(1);
    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();
    assert!(id > 0);
}

#[test]
fn test_save_snapshot_in_memory() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();
    assert!(id > 0);
}

#[test]
fn test_save_snapshot_increments_id() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id1 = storage.save_snapshot(&dummy_snapshot()).unwrap();
    let id2 = storage.save_snapshot(&dummy_snapshot()).unwrap();

    assert!(id2 > id1);
}

#[test]
fn test_load_latest_snapshot_empty() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let result = storage.load_latest_snapshot().unwrap();
    assert!(result.is_none());
}

#[test]
fn test_load_latest_snapshot_single() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let snap = dummy_snapshot();
    storage.save_snapshot(&snap).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.movement.current_room_id, "start");
}

#[tokio::test]
async fn test_load_latest_snapshot_most_recent() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let mut snap1 = dummy_snapshot();
    snap1.movement.current_room_id = "room1".to_string();
    storage.save_snapshot(&snap1).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut snap2 = dummy_snapshot();
    snap2.movement.current_room_id = "room2".to_string();
    storage.save_snapshot(&snap2).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.movement.current_room_id, "room2");
}

#[test]
fn test_load_latest_snapshot_excludes_other_games() {
    let storage = Storage::new_in_memory();

    storage.set_game_id(1);
    let mut snap1 = dummy_snapshot();
    snap1.movement.current_room_id = "game1".to_string();
    storage.save_snapshot(&snap1).unwrap();

    storage.set_game_id(2);
    let mut snap2 = dummy_snapshot();
    snap2.movement.current_room_id = "game2".to_string();
    storage.save_snapshot(&snap2).unwrap();

    storage.set_game_id(1);
    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.movement.current_room_id, "game1");
}

#[test]
fn test_load_snapshot_by_id_found() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();

    let loaded = storage.load_snapshot_by_id(id).unwrap().unwrap();
    assert_eq!(loaded.db_id, Some(id));
}

#[test]
fn test_load_snapshot_by_id_not_found() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);
    let result = storage.load_snapshot_by_id(9999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_load_snapshot_by_id_excludes_other_games() {
    let storage = Storage::new_in_memory();

    storage.set_game_id(1);
    let _id1 = storage.save_snapshot(&dummy_snapshot()).unwrap();

    storage.set_game_id(2);
    let id2 = storage.save_snapshot(&dummy_snapshot()).unwrap();

    storage.set_game_id(1);
    let result = storage.load_snapshot_by_id(id2).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_snapshot_movement_field() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let mut snap = dummy_snapshot();
    snap.movement.current_room_id = "test_room".to_string();
    storage.save_snapshot(&snap).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.movement.current_room_id, "test_room");
}

#[test]
fn test_snapshot_narrative_field() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let mut snap = dummy_snapshot();
    snap.narrative.pending_location = Some("Castle".to_string());
    snap.narrative.pending_event = Some("Battle".to_string());
    storage.save_snapshot(&snap).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(
        loaded.narrative.pending_location,
        Some("Castle".to_string())
    );
    assert_eq!(loaded.narrative.pending_event, Some("Battle".to_string()));
}

#[test]
fn test_snapshot_scene_field() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let snap = dummy_snapshot();
    storage.save_snapshot(&snap).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.scene, SceneState::default());
}

#[test]
fn test_snapshot_npc_encounter_log() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let snap = dummy_snapshot();
    storage.save_snapshot(&snap).unwrap();

    let loaded = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(loaded.npc_encounter_log, NpcEncounterLog::default());
}

#[test]
fn test_snapshot_created_at_timestamp() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let before = chrono::Utc::now();
    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();
    let after = chrono::Utc::now();

    let loaded = storage.load_snapshot_by_id(id).unwrap().unwrap();
    assert!(loaded.created_at >= before);
    assert!(loaded.created_at <= after);
}

#[test]
fn test_snapshot_db_id_assigned() {
    let storage = Storage::new_in_memory();
    storage.set_game_id(1);

    let id = storage.save_snapshot(&dummy_snapshot()).unwrap();
    let loaded = storage.load_snapshot_by_id(id).unwrap().unwrap();

    assert_eq!(loaded.db_id, Some(id));
}

#[test]
fn test_save_snapshot_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);

    handle.set("save_snapshot", TestOverride::internal("save failed"));

    let result = storage.save_snapshot(&dummy_snapshot());
    assert!(result.is_err());
}

#[test]
fn test_load_latest_snapshot_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);

    handle.set("load_latest_snapshot", TestOverride::config("load failed"));

    let result = storage.load_latest_snapshot();
    assert!(result.is_err());
}

#[test]
fn test_load_snapshot_by_id_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    storage.set_game_id(1);

    handle.set(
        "load_snapshot_by_id",
        TestOverride::internal("load by id failed"),
    );

    let result = storage.load_snapshot_by_id(1);
    assert!(result.is_err());
}
