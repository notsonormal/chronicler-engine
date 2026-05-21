use chrono::Utc;

use crate::model::checkpoint::Checkpoint;
use crate::model::state::{MovementState, SceneState};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::state_snapshot::NarrativeSnapshot;
use crate::model::trigger::NpcEncounterLog;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::in_memory_storage::InMemoryGameStorage;

fn empty_snapshot() -> GameStateSnapshot {
    GameStateSnapshot {
        db_id: None,

        movement: MovementState {
            current_room_id: String::new(),
            dynamic_rooms: std::collections::HashMap::new(),
        },
        narrative: NarrativeSnapshot::default(),
        scene: SceneState {
            npcs_in_area: Vec::new(),
            ..Default::default()
        },
        npc_encounter_log: NpcEncounterLog::default(),
        created_at: Utc::now(),
    }
}

#[test]
fn test_new_and_default() {
    let s = InMemoryGameStorage::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
    let d: InMemoryGameStorage = Default::default();
    assert!(d.is_empty());
}

#[test]
fn test_with_game_id() {
    let s = InMemoryGameStorage::with_game_id(42);
    assert!(s.is_empty());
}

#[test]
fn test_save_and_load_latest() {
    let s = InMemoryGameStorage::new();
    let snap = empty_snapshot();
    let id = s.save(&snap).unwrap();
    assert_eq!(s.len(), 1);
    let loaded = s.load_latest().unwrap().unwrap();
    assert_eq!(loaded.db_id, Some(id));
}

#[test]
fn test_load_by_id() {
    let s = InMemoryGameStorage::new();
    let id = s.save(&empty_snapshot()).unwrap();
    assert!(s.load_by_id(id).unwrap().is_some());
    assert!(s.load_by_id(999).unwrap().is_none());
}

#[test]
fn test_reset() {
    let s = InMemoryGameStorage::new();
    s.save(&empty_snapshot()).unwrap();
    s.reset().unwrap();
    assert!(s.is_empty());
}

#[test]
fn test_messages_roundtrip() {
    let s = InMemoryGameStorage::new();
    let mut msg = crate::model::message::Message {
        id: 1,
        sender: Some("A".to_string()),
        text: "hello".to_string(),
        log_type: crate::model::state::LogType::Input,
        timestamp: Utc::now(),
        location_header: None,
        event_header: None,
        snapshot_id: None,
    };
    s.insert_message(&mut msg).unwrap();
    let loaded = s.load_messages().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].text, "hello");
}

#[test]
fn test_checkpoint_roundtrip() {
    let s = InMemoryGameStorage::new();
    let cp = Checkpoint {
        id: "cp1".to_string(),
        snapshot_id: 1,
        name: "Test".to_string(),
        created_at: Utc::now(),
    };
    s.save_checkpoint(&cp).unwrap();
    let loaded = s.load_checkpoint("cp1").unwrap().unwrap();
    assert_eq!(loaded.name, "Test");
    let list = s.list_checkpoints().unwrap();
    assert_eq!(list.len(), 1);
    s.delete_checkpoint("cp1").unwrap();
    assert!(s.load_checkpoint("cp1").unwrap().is_none());
}
