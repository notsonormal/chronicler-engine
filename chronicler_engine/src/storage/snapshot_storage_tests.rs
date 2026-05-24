use chrono::Utc;

use crate::model::state::{MovementState, SceneState};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::state_snapshot::NarrativeSnapshot;
use crate::model::trigger::NpcEncounterLog;
use crate::storage::db::DbPool;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::{SnapshotStorage, SqliteSnapshotRepository};

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
fn test_sqlite_save_load_respects_game_id() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage_a = SqliteSnapshotRepository::new(pool.clone(), 1);
    let storage_b = SqliteSnapshotRepository::new(pool.clone(), 2);

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
    let storage_a = crate::storage::message_storage::SqliteMessageRepository::new(pool.clone(), 1);
    let storage_b = crate::storage::message_storage::SqliteMessageRepository::new(pool.clone(), 2);

    let msg_a = crate::model::message::Message::new(
        Some("A".to_string()),
        "hello",
        crate::model::state::LogType::Input,
        None,
        None,
    );
    let msg_b = crate::model::message::Message::new(
        Some("B".to_string()),
        "world",
        crate::model::state::LogType::Input,
        None,
        None,
    );

    storage_a.insert_message(&msg_a).unwrap();
    storage_b.insert_message(&msg_b).unwrap();

    let loaded_a = storage_a.load_messages().unwrap();
    assert_eq!(loaded_a.len(), 1);
    assert_eq!(loaded_a[0].text, "hello");

    let loaded_b = storage_b.load_messages().unwrap();
    assert_eq!(loaded_b.len(), 1);
    assert_eq!(loaded_b[0].text, "world");
}

#[test]
fn test_sqlite_load_by_id_not_found() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = SqliteSnapshotRepository::new(pool, 1);
    assert!(storage.load_by_id(999).unwrap().is_none());
}

#[test]
fn test_sqlite_load_latest_empty() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = SqliteSnapshotRepository::new(pool, 1);
    assert!(storage.load_latest().unwrap().is_none());
}
