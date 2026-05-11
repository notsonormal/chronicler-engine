mod test_data;

use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::storage::db::DbPool;
use chronicler_engine::storage::snapshot_storage::{SnapshotStorage, SqliteSnapshotStorage};

use test_data::create_test_state;

fn create_storage() -> SqliteSnapshotStorage {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    SqliteSnapshotStorage::new(pool)
}

fn create_snapshot(message_id: &str, swipe_index: u32) -> GameStateSnapshot {
    let state = create_test_state();
    GameStateSnapshot::from_game_state(&state, message_id.to_string(), swipe_index)
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
fn test_load_by_message_found() {
    let storage = create_storage();
    let snap = create_snapshot("msg1", 2);
    storage.save(&snap).unwrap();

    let loaded = storage
        .load_by_message("msg1", 2)
        .expect("load should succeed");
    assert!(
        loaded.is_some(),
        "should find snapshot by message_id and swipe_index"
    );
    assert_eq!(loaded.unwrap().message_id, "msg1");
}

#[test]
fn test_load_by_message_not_found() {
    let storage = create_storage();
    storage.save(&create_snapshot("msg1", 0)).unwrap();

    let loaded = storage
        .load_by_message("msg2", 0)
        .expect("load should succeed");
    assert!(
        loaded.is_none(),
        "should return None for missing message_id"
    );

    let loaded2 = storage
        .load_by_message("msg1", 99)
        .expect("load should succeed");
    assert!(
        loaded2.is_none(),
        "should return None for missing swipe_index"
    );
}

#[test]
fn test_load_latest_with_message_id_filter() {
    let storage = create_storage();
    let snap_a = create_snapshot("msg_a", 0);
    let snap_b = create_snapshot("msg_b", 0);
    storage.save(&snap_a).unwrap();
    storage.save(&snap_b).unwrap();

    let loaded = storage
        .load_latest(Some("msg_a"))
        .expect("load should succeed");
    assert_eq!(
        loaded.unwrap().message_id,
        "msg_a",
        "should filter by message_id"
    );
}

#[test]
fn test_save_updates_on_conflict() {
    let storage = create_storage();
    let mut snap = create_snapshot("msg1", 0);
    storage.save(&snap).unwrap();

    // Modify and save again with same message_id + swipe_index
    snap.committed = true;
    storage.save(&snap).unwrap();

    let loaded = storage
        .load_by_message("msg1", 0)
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
             (id, message_id, swipe_index, movement, narrative, scene, character_state, committed, created_at)
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
             (id, message_id, swipe_index, movement, narrative, scene, character_state, committed, created_at)
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
