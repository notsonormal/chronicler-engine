use chrono::Utc;

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
    let mut msg = crate::model::message::Message::new(
        Some("A".to_string()),
        "hello",
        crate::model::state::LogType::Input,
        None,
        None,
    );
    msg.id = 1;
    s.insert_message(&mut msg).unwrap();
    let loaded = s.load_messages().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].text, "hello");
}
