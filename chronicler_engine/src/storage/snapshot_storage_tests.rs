use chrono::Utc;

use crate::model::state::{MovementState, SceneState};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::state_snapshot::NarrativeSnapshot;
use crate::model::trigger::CharacterState;
use crate::storage::db::DbPool;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::{SnapshotStorage, SqliteGameStorage};

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
        },
        character_state: CharacterState::default(),
        committed: false,
        created_at: Utc::now(),
    }
}

#[test]
fn test_sqlite_save_load_respects_game_id() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage_a = SqliteGameStorage::new(pool.clone(), 1);
    let storage_b = SqliteGameStorage::new(pool.clone(), 2);

    let mut snap_a = empty_snapshot();
    let id_a = storage_a.save(&snap_a).unwrap();
    snap_a.db_id = Some(id_a);

    let mut snap_b = empty_snapshot();
    let id_b = storage_b.save(&snap_b).unwrap();
    snap_b.db_id = Some(id_b);

    let loaded_a = storage_a.load_latest().unwrap().unwrap();
    assert_eq!(loaded_a.db_id, Some(id_a));

    let loaded_b = storage_b.load_latest().unwrap().unwrap();
    assert_eq!(loaded_b.db_id, Some(id_b));
}

#[test]
fn test_sqlite_messages_respect_game_id() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage_a = SqliteGameStorage::new(pool.clone(), 1);
    let storage_b = SqliteGameStorage::new(pool.clone(), 2);

    let mut msg_a = crate::model::message::Message {
        id: 1,
        sender: Some("A".to_string()),
        text: "hello".to_string(),
        log_type: crate::model::state::LogType::Input,
        timestamp: Utc::now(),
        location_header: None,
        event_header: None,
        snapshot_id: None,
    };
    let mut msg_b = crate::model::message::Message {
        id: 2,
        sender: Some("B".to_string()),
        text: "world".to_string(),
        log_type: crate::model::state::LogType::Input,
        timestamp: Utc::now(),
        location_header: None,
        event_header: None,
        snapshot_id: None,
    };

    storage_a.insert_message(&mut msg_a).unwrap();
    storage_b.insert_message(&mut msg_b).unwrap();

    let loaded_a = storage_a.load_messages().unwrap();
    assert_eq!(loaded_a.len(), 1);
    assert_eq!(loaded_a[0].text, "hello");

    let loaded_b = storage_b.load_messages().unwrap();
    assert_eq!(loaded_b.len(), 1);
    assert_eq!(loaded_b[0].text, "world");
}

#[test]
fn test_sqlite_reset_clears_only_current_game() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage_a = SqliteGameStorage::new(pool.clone(), 1);
    let storage_b = SqliteGameStorage::new(pool.clone(), 2);

    let snap = empty_snapshot();
    storage_a.save(&snap).unwrap();
    storage_b.save(&snap).unwrap();

    storage_a.reset().unwrap();

    assert!(storage_a.load_latest().unwrap().is_none());
    assert!(storage_b.load_latest().unwrap().is_some());
}

#[test]
fn test_sqlite_commit() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = SqliteGameStorage::new(pool, 1);

    let snap = empty_snapshot();
    let id = storage.save(&snap).unwrap();

    storage.commit(id).unwrap();

    let loaded = storage.load_by_id(id).unwrap().unwrap();
    assert!(loaded.committed);
}

#[test]
fn test_sqlite_checkpoint_roundtrip() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = SqliteGameStorage::new(pool, 1);

    let snap = empty_snapshot();
    let id = storage.save(&snap).unwrap();

    let cp = crate::model::checkpoint::Checkpoint {
        id: "cp1".to_string(),
        snapshot_id: id,
        name: "Test Checkpoint".to_string(),
        created_at: Utc::now(),
    };

    storage.save_checkpoint(&cp).unwrap();

    let loaded = storage.load_checkpoint("cp1").unwrap().unwrap();
    assert_eq!(loaded.snapshot_id, id);
    assert_eq!(loaded.name, "Test Checkpoint");

    let list = storage.list_checkpoints().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "cp1");

    storage.delete_checkpoint("cp1").unwrap();
    assert!(storage.load_checkpoint("cp1").unwrap().is_none());
}

#[test]
fn test_sqlite_load_by_id_not_found() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = SqliteGameStorage::new(pool, 1);
    assert!(storage.load_by_id(999).unwrap().is_none());
}

#[test]
fn test_sqlite_load_latest_empty() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = SqliteGameStorage::new(pool, 1);
    assert!(storage.load_latest().unwrap().is_none());
}
