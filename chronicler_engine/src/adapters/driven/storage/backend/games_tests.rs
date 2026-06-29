use crate::adapters::driven::storage::backend::{Storage, TestOverride};
use crate::test_support::sqlite_storage;

#[test]
fn test_create_game_returns_positive_id() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "Game A")
        .unwrap();
    assert!(id > 0);
}

#[test]
fn test_create_game_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "Game A")
        .unwrap();
    assert!(id > 0);

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.world_name, "test");
    assert_eq!(game.name, "Game A");
}

#[test]
fn test_create_game_in_memory() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "Game A")
        .unwrap();
    assert!(id > 0);

    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.world_name, "test");
    assert_eq!(game.name, "Game A");
}

#[test]
fn test_get_game_found() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "G")
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.id, id);
}

#[test]
fn test_get_game_not_found() {
    let storage = Storage::new_in_memory();
    let game = storage.get_game(9999).unwrap();
    assert!(game.is_none());
}

#[test]
fn test_get_game_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .create_game("test", "test", "test_player", "Test Player", "SQLiteGame")
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();
    assert_eq!(game.name, "SQLiteGame");
}

#[test]
fn test_delete_game_existing() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "ToDelete")
        .unwrap();
    storage.delete_game(id).unwrap();

    let game = storage.get_game(id).unwrap();
    assert!(game.is_none());
}

#[test]
fn test_delete_game_nonexistent() {
    let storage = Storage::new_in_memory();
    let result = storage.delete_game(9999);
    assert!(result.is_ok());
}

#[test]
fn test_delete_game_sqlite() {
    let storage = sqlite_storage().unwrap();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "ToDelete")
        .unwrap();
    storage.delete_game(id).unwrap();

    let game = storage.get_game(id).unwrap();
    assert!(game.is_none());
}

#[test]
fn test_game_timestamps_created_at() {
    let storage = Storage::new_in_memory();
    let id = storage
        .create_game("w", "w", "test_player", "Test Player", "G")
        .unwrap();
    let game = storage.get_game(id).unwrap().unwrap();

    let now = chrono::Utc::now();
    let diff = now - game.created_at;
    assert!(diff.num_seconds() < 2);
}

#[test]
fn test_create_game_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("create_game", TestOverride::internal("create failed"));

    let result = storage.create_game("w", "w", "test_player", "Test Player", "G");
    assert!(result.is_err());
}

#[test]
fn test_get_game_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("get_game", TestOverride::config("get failed"));

    let result = storage.get_game(1);
    assert!(result.is_err());
}

#[test]
fn test_delete_game_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("delete_game", TestOverride::config("delete failed"));

    let result = storage.delete_game(1);
    assert!(result.is_err());
}
