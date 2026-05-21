mod test_data;

use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::storage::db::DbPool;
use chronicler_engine::storage::snapshot_storage::{SnapshotStorage, SqliteGameStorage};

use test_data::create_test_state;

fn create_storage() -> SqliteGameStorage {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    SqliteGameStorage::new(pool, 1)
}

fn create_snapshot() -> GameStateSnapshot {
    let state = create_test_state();
    GameStateSnapshot::from_game_state(&state)
}

#[test]
fn test_save_creates_snapshot() {
    let storage = create_storage();
    let snap = create_snapshot();

    let snap_id = storage.save(&snap).expect("save should succeed");

    let loaded = storage.load_latest().expect("load should succeed").unwrap();
    assert_eq!(loaded.db_id, Some(snap_id), "save should create a snapshot");
}

#[test]
fn test_reset_deletes_all_snapshots() {
    let storage = create_storage();
    storage.save(&create_snapshot()).unwrap();
    storage.save(&create_snapshot()).unwrap();

    storage.reset().expect("reset should succeed");

    let loaded = storage.load_latest().expect("load should succeed");
    assert!(
        loaded.is_none(),
        "load_latest should return None after reset"
    );
}

#[test]
fn test_load_by_id_found() {
    let storage = create_storage();
    let snap = create_snapshot();
    let id = storage.save(&snap).unwrap();

    let loaded = storage.load_by_id(id).expect("load should succeed");
    assert!(loaded.is_some(), "should find snapshot by id");
    assert_eq!(loaded.unwrap().db_id, Some(id));
}

#[test]
fn test_load_by_id_not_found() {
    let storage = create_storage();
    storage.save(&create_snapshot()).unwrap();

    let loaded = storage.load_by_id(9999).expect("load should succeed");
    assert!(loaded.is_none(), "should return None for missing id");
}

#[test]
fn test_load_latest_returns_most_recent() {
    let storage = create_storage();
    let snap_a = create_snapshot();
    let snap_b = create_snapshot();
    storage.save(&snap_a).unwrap();
    let id_b = storage.save(&snap_b).unwrap();

    let loaded = storage.load_latest().expect("load should succeed");
    assert_eq!(
        loaded.unwrap().db_id,
        Some(id_b),
        "should return most recent snapshot"
    );
}

#[test]
fn test_save_creates_new_snapshot() {
    let storage = create_storage();
    let snap = create_snapshot();
    let id1 = storage.save(&snap).unwrap();

    let snap2 = create_snapshot();
    let id2 = storage.save(&snap2).unwrap();

    let loaded = storage
        .load_by_id(id2)
        .expect("load should succeed")
        .unwrap();
    assert_eq!(
        loaded.db_id,
        Some(id2),
        "second save should create new snapshot"
    );
    assert_ne!(id1, id2, "each save should get a unique id");
}

#[test]
fn test_row_to_snapshot_bad_json() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    {
        let conn = pool.conn();
        // Insert a row with invalid JSON in the movement column
        conn.execute(
            "INSERT INTO game_state_snapshots
             (movement, narrative, scene, npc_encounter_log, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "not valid json",
                "{}",
                "{}",
                "{}",
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .expect("raw insert should succeed");
    }

    let storage = SqliteGameStorage::new(pool, 1);
    let result = storage.load_latest();
    assert!(result.is_err(), "loading bad JSON should return an error");
}

#[test]
fn test_row_to_snapshot_bad_date() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    {
        let conn = pool.conn();
        conn.execute(
            "INSERT INTO game_state_snapshots
             (movement, narrative, scene, npc_encounter_log, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["{}", "{}", "{}", "{}", "not-a-date",],
        )
        .expect("raw insert should succeed");
    }

    let storage = SqliteGameStorage::new(pool, 1);
    let result = storage.load_latest();
    assert!(result.is_err(), "loading bad date should return an error");
}

#[test]
fn test_checkpoint_crud() {
    use chronicler_engine::model::checkpoint::Checkpoint;
    let storage = create_storage();

    let cp = Checkpoint {
        id: "cp1".to_string(),
        snapshot_id: 42,
        name: "Test Checkpoint".to_string(),
        created_at: chrono::Utc::now(),
    };

    storage.save_checkpoint(&cp).expect("save should succeed");

    let loaded = storage
        .load_checkpoint("cp1")
        .expect("load should succeed")
        .expect("checkpoint should exist");
    assert_eq!(loaded.id, "cp1");
    assert_eq!(loaded.snapshot_id, 42);
    assert_eq!(loaded.name, "Test Checkpoint");

    let list = storage.list_checkpoints().expect("list should succeed");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "cp1");

    storage
        .delete_checkpoint("cp1")
        .expect("delete should succeed");
    let after_delete = storage.load_checkpoint("cp1").expect("load should succeed");
    assert!(after_delete.is_none(), "checkpoint should be deleted");
}

#[test]
fn test_checkpoint_upsert() {
    use chronicler_engine::model::checkpoint::Checkpoint;
    let storage = create_storage();

    let mut cp = Checkpoint {
        id: "cp1".to_string(),
        snapshot_id: 1,
        name: "Original".to_string(),
        created_at: chrono::Utc::now(),
    };
    storage.save_checkpoint(&cp).unwrap();

    cp.snapshot_id = 3;
    cp.name = "Updated".to_string();
    storage.save_checkpoint(&cp).unwrap();

    let loaded = storage.load_checkpoint("cp1").unwrap().unwrap();
    assert_eq!(loaded.snapshot_id, 3);
    assert_eq!(loaded.name, "Updated");
}

#[test]
fn test_reset_clears_checkpoints() {
    use chronicler_engine::model::checkpoint::Checkpoint;
    let storage = create_storage();

    let cp = Checkpoint {
        id: "cp1".to_string(),
        snapshot_id: 1,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    storage.save_checkpoint(&cp).unwrap();
    storage.reset().expect("reset should succeed");

    let list = storage.list_checkpoints().expect("list should succeed");
    assert!(list.is_empty(), "checkpoints should be cleared after reset");
}
