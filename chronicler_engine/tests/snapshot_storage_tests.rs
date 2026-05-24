mod test_data;

use chronicler_engine::model::message::Message;
use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::storage::db::DbPool;
use chronicler_engine::storage::message_storage::MessageStorage;
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

// ─── Game CRUD ──────────────────────────────────────────────────────────────

#[test]
fn test_create_and_get_game() {
    let storage = create_storage();

    let game_id = storage.create_game("test_world", "My Game").unwrap();
    assert!(game_id > 0, "create_game should return a positive id");

    let game = storage.get_game(game_id).unwrap();
    assert!(game.is_some(), "get_game should find the created game");
    let game = game.unwrap();
    assert_eq!(game.world_name, "test_world");
    assert_eq!(game.name, "My Game");
}

#[test]
fn test_get_game_not_found() {
    let storage = create_storage();

    let game = storage.get_game(9999).unwrap();
    assert!(game.is_none(), "get_game should return None for missing id");
}

#[test]
fn test_list_games() {
    let storage = create_storage();
    let initial = storage.list_games().unwrap().len();

    let id_a = storage.create_game("world_a", "Game A").unwrap();
    let id_b = storage.create_game("world_b", "Game B").unwrap();

    let games = storage.list_games().unwrap();
    assert_eq!(
        games.len(),
        initial + 2,
        "list_games should return both new games"
    );

    // Most recently updated first
    assert_eq!(games[0].id, id_b);
    assert_eq!(games[1].id, id_a);
}

#[test]
fn test_delete_game_cascades() {
    let storage = create_storage();
    let game_id = storage.create_game("test_world", "To Delete").unwrap();

    // Switch to the new game before saving data
    SnapshotStorage::set_game_id(&storage, game_id);

    // Save a snapshot and message for this game
    storage.save(&create_snapshot()).unwrap();
    let mut msg = Message::new(
        Some("Player".to_string()),
        "hello",
        chronicler_engine::model::state::LogType::Input,
        None,
        None,
    );
    storage.insert_message(&mut msg).unwrap();

    storage.delete_game(game_id).expect("delete should succeed");

    assert!(
        storage.get_game(game_id).unwrap().is_none(),
        "game should be deleted"
    );
    assert!(
        storage.load_latest().unwrap().is_none(),
        "snapshots should be cascaded"
    );
    assert!(
        storage.load_messages().unwrap().is_empty(),
        "messages should be cascaded"
    );
}

// ─── Game ID Switching ──────────────────────────────────────────────────────

#[test]
fn test_set_game_id_isolates_snapshots() {
    let storage = create_storage();

    let game_a = storage.create_game("world_a", "Game A").unwrap();
    let game_b = storage.create_game("world_b", "Game B").unwrap();

    // Save snapshot for game_a (current default game_id is 1, not game_a)
    SnapshotStorage::set_game_id(&storage, game_a);
    let id_a = storage.save(&create_snapshot()).unwrap();

    // Save snapshot for game_b
    SnapshotStorage::set_game_id(&storage, game_b);
    let id_b = storage.save(&create_snapshot()).unwrap();

    // Load latest for game_b
    let latest_b = storage.load_latest().unwrap().unwrap();
    assert_eq!(latest_b.db_id, Some(id_b));

    // Switch back to game_a
    SnapshotStorage::set_game_id(&storage, game_a);
    let latest_a = storage.load_latest().unwrap().unwrap();
    assert_eq!(latest_a.db_id, Some(id_a));

    // game_id 1 (default) should have no snapshots
    SnapshotStorage::set_game_id(&storage, 1);
    assert!(
        storage.load_latest().unwrap().is_none(),
        "default game_id should have no snapshots"
    );
}

#[test]
fn test_current_game_id() {
    let storage = create_storage();
    assert_eq!(SnapshotStorage::current_game_id(&storage), 1);

    SnapshotStorage::set_game_id(&storage, 42);
    assert_eq!(SnapshotStorage::current_game_id(&storage), 42);
}

// ─── Message Storage ────────────────────────────────────────────────────────

#[test]
fn test_insert_and_load_messages() {
    let storage = create_storage();

    let mut msg = Message::new(
        Some("Player".to_string()),
        "look around",
        chronicler_engine::model::state::LogType::Input,
        None,
        None,
    );

    storage.insert_message(&mut msg).unwrap();
    assert!(msg.id > 0, "insert_message should set the message id");

    let loaded = storage.load_messages().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].text, "look around");
}

#[test]
fn test_update_message() {
    let storage = create_storage();

    let mut msg = Message::new(
        Some("Player".to_string()),
        "original",
        chronicler_engine::model::state::LogType::Input,
        None,
        None,
    );
    storage.insert_message(&mut msg).unwrap();

    storage.update_message(msg.id, "edited").unwrap();

    let loaded = storage.load_messages().unwrap();
    assert_eq!(loaded[0].text, "edited");
}

#[test]
fn test_delete_message() {
    let storage = create_storage();

    let mut msg = Message::new(
        Some("Player".to_string()),
        "to delete",
        chronicler_engine::model::state::LogType::Input,
        None,
        None,
    );
    storage.insert_message(&mut msg).unwrap();
    storage.delete_message(msg.id).unwrap();

    let loaded = storage.load_messages().unwrap();
    assert!(loaded.is_empty(), "message should be deleted");
}

#[test]
fn test_load_messages_empty() {
    let storage = create_storage();
    let loaded = storage.load_messages().unwrap();
    assert!(
        loaded.is_empty(),
        "load_messages should return empty vec when no messages"
    );
}

// ─── Edge Cases ─────────────────────────────────────────────────────────────

#[test]
fn test_reset_empty_storage() {
    let storage = create_storage();
    // Resetting an empty storage should not error
    storage
        .reset()
        .expect("reset on empty storage should succeed");
    assert!(storage.load_latest().unwrap().is_none());
}

#[test]
fn test_load_latest_no_snapshots() {
    let storage = create_storage();
    let latest = storage.load_latest().unwrap();
    assert!(
        latest.is_none(),
        "load_latest should return None when no snapshots exist"
    );
}
