//! Integration tests for game-state snapshot persistence: save/load, missing-snapshot errors, and message/swipe round-tripping against a real SQLite-backed `Storage`.

use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::db::DbPool;

use crate::fixtures::{create_test_state, create_test_storage};
fn create_storage() -> Storage {
    create_test_storage(1)
}

fn create_game_storage() -> Storage {
    create_test_storage(1)
}

fn create_snapshot() -> GameStateSnapshot {
    let state = create_test_state();
    GameStateSnapshot::from_game_state(&state)
}

#[test]
fn test_save_creates_snapshot() {
    let storage = create_storage();
    let snap = create_snapshot();

    let snap_id = storage.save_snapshot(&snap).expect("save should succeed");

    let loaded = storage
        .load_latest_snapshot()
        .expect("load should succeed")
        .unwrap();
    assert_eq!(loaded.db_id, Some(snap_id), "save should create a snapshot");
}

#[test]
fn test_load_by_id_found() {
    let storage = create_storage();
    let snap = create_snapshot();
    let id = storage.save_snapshot(&snap).unwrap();

    let loaded = storage
        .load_snapshot_by_id(id)
        .expect("load should succeed");
    assert!(loaded.is_some(), "should find snapshot by id");
    assert_eq!(loaded.unwrap().db_id, Some(id));
}

#[test]
fn test_load_by_id_not_found() {
    let storage = create_storage();
    storage.save_snapshot(&create_snapshot()).unwrap();

    let loaded = storage
        .load_snapshot_by_id(9999)
        .expect("load should succeed");
    assert!(loaded.is_none(), "should return None for missing id");
}

#[test]
fn test_load_latest_returns_most_recent() {
    let storage = create_storage();
    let snap_a = create_snapshot();
    let snap_b = create_snapshot();
    storage.save_snapshot(&snap_a).unwrap();
    let id_b = storage.save_snapshot(&snap_b).unwrap();

    let loaded = storage.load_latest_snapshot().expect("load should succeed");
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
    let id1 = storage.save_snapshot(&snap).unwrap();

    let snap2 = create_snapshot();
    let id2 = storage.save_snapshot(&snap2).unwrap();

    let loaded = storage
        .load_snapshot_by_id(id2)
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
    chronicler_engine::test_support::seed_default_game_row(&pool, 1).unwrap();
    {
        let conn = pool.conn();
        conn.execute(
            "INSERT INTO game_state_snapshots
             (game_id, movement, narrative, scene, npc_encounter_log, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                1i64,
                "not valid json",
                "{}",
                "{}",
                "{}",
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .expect("raw insert should succeed");
    }

    let storage = Storage::new_sqlite(pool, 1);
    let result = storage.load_latest_snapshot();
    assert!(result.is_err(), "loading bad JSON should return an error");
}

#[test]
fn test_row_to_snapshot_bad_date() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    chronicler_engine::test_support::seed_default_game_row(&pool, 1).unwrap();
    {
        let conn = pool.conn();
        conn.execute(
            "INSERT INTO game_state_snapshots
             (game_id, movement, narrative, scene, npc_encounter_log, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![1i64, "{}", "{}", "{}", "{}", "not-a-date",],
        )
        .expect("raw insert should succeed");
    }

    let storage = Storage::new_sqlite(pool, 1);
    let result = storage.load_latest_snapshot();
    assert!(result.is_err(), "loading bad date should return an error");
}

#[test]
fn test_create_and_get_game() {
    let storage = create_game_storage();

    let game_id = storage
        .create_game(
            "test_world",
            "test_world",
            "test_player",
            "Test Player",
            "My Game",
        )
        .unwrap();
    assert!(game_id > 0, "create_game should return a positive id");

    let game = storage.get_game(game_id).unwrap();
    assert!(game.is_some(), "get_game should find the created game");
    let game = game.unwrap();
    assert_eq!(game.world_name, "test_world");
    assert_eq!(game.name, "My Game");
}

#[test]
fn test_get_game_not_found() {
    let storage = create_game_storage();

    let game = storage.get_game(9999).unwrap();
    assert!(game.is_none(), "get_game should return None for missing id");
}

#[test]
fn test_list_games() {
    let storage = create_game_storage();
    let initial = storage.list_games().unwrap().len();

    let id_a = storage
        .create_game("world_a", "world_a", "test_player", "Test Player", "Game A")
        .unwrap();
    let id_b = storage
        .create_game("world_b", "world_b", "test_player", "Test Player", "Game B")
        .unwrap();

    let games = storage.list_games().unwrap();
    assert_eq!(
        games.len(),
        initial + 2,
        "list_games should return both new games"
    );

    assert_eq!(games[0].id, id_b);
    assert_eq!(games[1].id, id_a);
}

#[test]
fn test_delete_game_cascades() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    let storage = Storage::new_sqlite(pool.clone(), 1);
    let msg_storage = Storage::new_sqlite(pool, 1);

    let game_id = storage
        .create_game(
            "test_world",
            "test_world",
            "test_player",
            "Test Player",
            "To Delete",
        )
        .unwrap();

    storage.set_game_id(game_id);
    msg_storage.set_game_id(game_id);

    storage.save_snapshot(&create_snapshot()).unwrap();
    let msg = Message::new(
        Some("Player".to_string()),
        "hello",
        MessageType::Input,
        None,
        None,
    );
    msg_storage.insert_message(&msg).unwrap();

    storage.delete_game(game_id).expect("delete should succeed");

    assert!(
        storage.get_game(game_id).unwrap().is_none(),
        "game should be deleted"
    );
    assert!(
        storage.load_latest_snapshot().unwrap().is_none(),
        "snapshots should be cascaded"
    );
    assert!(
        msg_storage.load_message_rows().unwrap().is_empty(),
        "messages should be cascaded"
    );
}

#[test]
fn test_set_game_id_isolates_snapshots() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    let game_storage = Storage::new_sqlite(pool.clone(), 1);
    let storage = Storage::new_sqlite(pool, 1);

    let game_a = game_storage
        .create_game("world_a", "world_a", "test_player", "Test Player", "Game A")
        .unwrap();
    let game_b = game_storage
        .create_game("world_b", "world_b", "test_player", "Test Player", "Game B")
        .unwrap();

    storage.set_game_id(game_a);
    let id_a = storage.save_snapshot(&create_snapshot()).unwrap();

    storage.set_game_id(game_b);
    let id_b = storage.save_snapshot(&create_snapshot()).unwrap();

    let latest_b = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(latest_b.db_id, Some(id_b));

    storage.set_game_id(game_a);
    let latest_a = storage.load_latest_snapshot().unwrap().unwrap();
    assert_eq!(latest_a.db_id, Some(id_a));

    storage.set_game_id(999);
    assert!(
        storage.load_latest_snapshot().unwrap().is_none(),
        "default game_id should have no snapshots"
    );
}

#[test]
fn test_current_game_id() {
    let storage = create_storage();
    assert_eq!(storage.current_game_id(), 1);

    storage.set_game_id(42);
    assert_eq!(storage.current_game_id(), 42);
}

#[test]
fn test_insert_and_load_messages() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    chronicler_engine::test_support::seed_default_game_row(&pool, 1).unwrap();
    let msg_repo = Storage::new_sqlite(pool, 1);

    let msg = Message::new(
        Some("Player".to_string()),
        "look around",
        MessageType::Input,
        None,
        None,
    );

    let id = msg_repo.insert_message(&msg).unwrap();
    assert!(id > 0, "insert_message should return a message id");

    let loaded = msg_repo.load_message_rows().unwrap();
    assert_eq!(loaded.len(), 1);
}

#[test]
fn test_get_and_update_active_swipe_index() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    chronicler_engine::test_support::seed_default_game_row(&pool, 1).unwrap();
    let msg_repo = Storage::new_sqlite(pool, 1);

    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        MessageType::Input,
        None,
        None,
    );
    let id = msg_repo.insert_message(&msg).unwrap();

    assert_eq!(msg_repo.get_active_swipe_index(id).unwrap(), Some(0));
    msg_repo.update_active_swipe(id, 2).unwrap();
    assert_eq!(msg_repo.get_active_swipe_index(id).unwrap(), Some(2));
}

#[test]
fn test_delete_message() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    chronicler_engine::test_support::seed_default_game_row(&pool, 1).unwrap();
    let msg_repo = Storage::new_sqlite(pool, 1);

    let msg = Message::new(
        Some("Player".to_string()),
        "to delete",
        MessageType::Input,
        None,
        None,
    );
    let id = msg_repo.insert_message(&msg).unwrap();
    msg_repo.delete_message(id).unwrap();

    let loaded = msg_repo.load_message_rows().unwrap();
    assert!(loaded.is_empty(), "message should be deleted");
}

#[test]
fn test_load_messages_empty() {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    let msg_repo = Storage::new_sqlite(pool, 1);
    let loaded = msg_repo.load_message_rows().unwrap();
    assert!(
        loaded.is_empty(),
        "load_message_rows should return empty vec when no messages"
    );
}

#[test]
fn test_load_latest_no_snapshots() {
    let storage = create_storage();
    let latest = storage.load_latest_snapshot().unwrap();
    assert!(
        latest.is_none(),
        "load_latest should return None when no snapshots exist"
    );
}
