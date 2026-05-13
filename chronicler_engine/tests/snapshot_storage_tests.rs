mod test_data;

use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::storage::db::DbPool;
use chronicler_engine::storage::snapshot_storage::{SnapshotStorage, SqliteSnapshotStorage};

use test_data::create_test_state;

fn create_storage() -> SqliteSnapshotStorage {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    SqliteSnapshotStorage::new(pool)
}

fn create_snapshot(turn_id: &str, swipe_index: u32) -> GameStateSnapshot {
    let state = create_test_state();
    GameStateSnapshot::from_game_state(&state, turn_id.to_string(), swipe_index)
}

#[test]
fn test_commit_sets_committed_flag() {
    let storage = create_storage();
    let snap = create_snapshot("msg1", 0);
    let snap_id = snap.id.clone();

    storage.save(&snap).expect("save should succeed");

    let loaded_before = storage
        .load_latest(None)
        .expect("load should succeed")
        .unwrap();
    assert!(
        !loaded_before.committed,
        "new snapshot should not be committed"
    );

    storage.commit(&snap_id).expect("commit should succeed");

    let loaded_after = storage
        .load_latest(None)
        .expect("load should succeed")
        .unwrap();
    assert!(
        loaded_after.committed,
        "committed should be true after commit"
    );
}

#[test]
fn test_reset_deletes_all_snapshots() {
    let storage = create_storage();
    storage.save(&create_snapshot("msg1", 0)).unwrap();
    storage.save(&create_snapshot("msg2", 0)).unwrap();

    storage.reset().expect("reset should succeed");

    let loaded = storage.load_latest(None).expect("load should succeed");
    assert!(
        loaded.is_none(),
        "load_latest should return None after reset"
    );
}

#[test]
fn test_load_by_turn_found() {
    let storage = create_storage();
    let snap = create_snapshot("msg1", 2);
    storage.save(&snap).unwrap();

    let loaded = storage
        .load_by_turn("msg1", 2)
        .expect("load should succeed");
    assert!(
        loaded.is_some(),
        "should find snapshot by turn_id and swipe_index"
    );
    assert_eq!(loaded.unwrap().turn_id, "msg1");
}

#[test]
fn test_load_by_turn_not_found() {
    let storage = create_storage();
    storage.save(&create_snapshot("msg1", 0)).unwrap();

    let loaded = storage
        .load_by_turn("msg2", 0)
        .expect("load should succeed");
    assert!(loaded.is_none(), "should return None for missing turn_id");

    let loaded2 = storage
        .load_by_turn("msg1", 99)
        .expect("load should succeed");
    assert!(
        loaded2.is_none(),
        "should return None for missing swipe_index"
    );
}

#[test]
fn test_load_latest_with_turn_id_filter() {
    let storage = create_storage();
    let snap_a = create_snapshot("msg_a", 0);
    let snap_b = create_snapshot("msg_b", 0);
    storage.save(&snap_a).unwrap();
    storage.save(&snap_b).unwrap();

    let loaded = storage
        .load_latest(Some("msg_a"))
        .expect("load should succeed");
    assert_eq!(loaded.unwrap().turn_id, "msg_a", "should filter by turn_id");
}

#[test]
fn test_save_updates_on_conflict() {
    let storage = create_storage();
    let mut snap = create_snapshot("msg1", 0);
    storage.save(&snap).unwrap();

    snap.committed = true;
    storage.save(&snap).unwrap();

    let loaded = storage
        .load_by_turn("msg1", 0)
        .expect("load should succeed")
        .unwrap();
    assert!(loaded.committed, "save on conflict should update committed");
}

#[test]
fn test_row_to_snapshot_bad_json() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    {
        let conn = pool.conn();
        // Insert a row with invalid JSON in the movement column
        conn.execute(
            "INSERT INTO game_state_snapshots
             (id, turn_id, swipe_index, movement, narrative, scene, character_state, committed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                "bad-id",
                "msg1",
                0,
                "not valid json",
                "{}",
                "{}",
                "{}",
                0,
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .expect("raw insert should succeed");
    }

    let storage = SqliteSnapshotStorage::new(pool);
    let result = storage.load_latest(None);
    assert!(result.is_err(), "loading bad JSON should return an error");
}

#[test]
fn test_row_to_snapshot_bad_date() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    {
        let conn = pool.conn();
        conn.execute(
            "INSERT INTO game_state_snapshots
             (id, turn_id, swipe_index, movement, narrative, scene, character_state, committed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                "bad-date-id",
                "msg1",
                0,
                "{}",
                "{}",
                "{}",
                "{}",
                0,
                "not-a-date",
            ],
        )
        .expect("raw insert should succeed");
    }

    let storage = SqliteSnapshotStorage::new(pool);
    let result = storage.load_latest(None);
    assert!(result.is_err(), "loading bad date should return an error");
}

#[test]
fn test_checkpoint_crud() {
    use chronicler_engine::model::checkpoint::Checkpoint;
    let storage = create_storage();

    let cp = Checkpoint {
        id: "cp1".to_string(),
        turn_id: "turn1".to_string(),
        swipe_index: 2,
        name: "Test Checkpoint".to_string(),
        created_at: chrono::Utc::now(),
    };

    storage.save_checkpoint(&cp).expect("save should succeed");

    let loaded = storage
        .load_checkpoint("cp1")
        .expect("load should succeed")
        .expect("checkpoint should exist");
    assert_eq!(loaded.id, "cp1");
    assert_eq!(loaded.turn_id, "turn1");
    assert_eq!(loaded.swipe_index, 2);
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
        turn_id: "turn1".to_string(),
        swipe_index: 0,
        name: "Original".to_string(),
        created_at: chrono::Utc::now(),
    };
    storage.save_checkpoint(&cp).unwrap();

    cp.swipe_index = 3;
    cp.name = "Updated".to_string();
    storage.save_checkpoint(&cp).unwrap();

    let loaded = storage.load_checkpoint("cp1").unwrap().unwrap();
    assert_eq!(loaded.swipe_index, 3);
    assert_eq!(loaded.name, "Updated");
}

#[test]
fn test_reset_clears_checkpoints() {
    use chronicler_engine::model::checkpoint::Checkpoint;
    let storage = create_storage();

    let cp = Checkpoint {
        id: "cp1".to_string(),
        turn_id: "turn1".to_string(),
        swipe_index: 0,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    storage.save_checkpoint(&cp).unwrap();
    storage.reset().expect("reset should succeed");

    let list = storage.list_checkpoints().expect("list should succeed");
    assert!(list.is_empty(), "checkpoints should be cleared after reset");
}
