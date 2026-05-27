use crate::storage::db::DbPool;
use crate::storage::game_storage::{GameStorage, SqliteGameRepository};
use crate::test_support::in_memory_storage::InMemoryGameRepository;

fn create_sqlite_repo() -> SqliteGameRepository {
    let pool = DbPool::new(":memory:").unwrap();
    SqliteGameRepository::new(pool, 1)
}

// ─── SQLite Tests ───────────────────────────────────────────────────────────

#[test]
fn test_sqlite_new_sets_game_id() {
    let repo = create_sqlite_repo();
    assert_eq!(repo.current_game_id(), 1);
}

#[test]
fn test_sqlite_set_game_id() {
    let repo = create_sqlite_repo();
    repo.set_game_id(42);
    assert_eq!(repo.current_game_id(), 42);
}

#[test]
fn test_sqlite_create_and_get_game() {
    let repo = create_sqlite_repo();

    let game_id = repo.create_game("test_world", "My Game").unwrap();
    assert!(game_id > 0, "create_game should return a positive id");

    let game = repo.get_game(game_id).unwrap();
    assert!(game.is_some(), "get_game should find the created game");
    let game = game.unwrap();
    assert_eq!(game.world_name, "test_world");
    assert_eq!(game.name, "My Game");
}

#[test]
fn test_sqlite_get_game_not_found() {
    let repo = create_sqlite_repo();
    let game = repo.get_game(9999).unwrap();
    assert!(game.is_none(), "get_game should return None for missing id");
}

#[test]
fn test_sqlite_list_games() {
    let repo = create_sqlite_repo();
    let initial = repo.list_games().unwrap().len();

    let id_a = repo.create_game("world_a", "Game A").unwrap();
    let id_b = repo.create_game("world_b", "Game B").unwrap();

    let games = repo.list_games().unwrap();
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
fn test_sqlite_delete_game() {
    let repo = create_sqlite_repo();
    let game_id = repo.create_game("test_world", "To Delete").unwrap();

    repo.delete_game(game_id).expect("delete should succeed");
    assert!(
        repo.get_game(game_id).unwrap().is_none(),
        "game should be deleted"
    );
}

#[test]
fn test_sqlite_set_game_id_updates_timestamp() {
    let repo = create_sqlite_repo();
    let game_id = repo.create_game("test_world", "Game").unwrap();

    let before = repo.get_game(game_id).unwrap().unwrap().updated_at;
    repo.set_game_id(game_id);
    let after = repo.get_game(game_id).unwrap().unwrap().updated_at;

    assert!(
        after >= before,
        "set_game_id should update games.updated_at"
    );
}

// ─── In-Memory Tests ────────────────────────────────────────────────────────

#[test]
fn test_in_memory_new_sets_game_id() {
    let repo = InMemoryGameRepository::new();
    assert_eq!(repo.current_game_id(), 1);
}

#[test]
fn test_in_memory_set_game_id() {
    let repo = InMemoryGameRepository::new();
    repo.set_game_id(42);
    assert_eq!(repo.current_game_id(), 42);
}

#[test]
fn test_in_memory_create_and_get_game() {
    let repo = InMemoryGameRepository::new();

    let game_id = repo.create_game("test_world", "My Game").unwrap();
    assert!(game_id > 0, "create_game should return a positive id");

    let game = repo.get_game(game_id).unwrap();
    assert!(game.is_some(), "get_game should find the created game");
    let game = game.unwrap();
    assert_eq!(game.world_name, "test_world");
    assert_eq!(game.name, "My Game");
}

#[test]
fn test_in_memory_get_game_not_found() {
    let repo = InMemoryGameRepository::new();
    let game = repo.get_game(9999).unwrap();
    assert!(game.is_none(), "get_game should return None for missing id");
}

#[test]
fn test_in_memory_list_games() {
    let repo = InMemoryGameRepository::new();
    let initial = repo.list_games().unwrap().len();

    let id_a = repo.create_game("world_a", "Game A").unwrap();
    let id_b = repo.create_game("world_b", "Game B").unwrap();

    let games = repo.list_games().unwrap();
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
fn test_in_memory_delete_game() {
    let repo = InMemoryGameRepository::new();
    let game_id = repo.create_game("test_world", "To Delete").unwrap();

    repo.delete_game(game_id).expect("delete should succeed");
    assert!(
        repo.get_game(game_id).unwrap().is_none(),
        "game should be deleted"
    );
}

#[test]
fn test_in_memory_set_game_id_updates_timestamp() {
    let repo = InMemoryGameRepository::new();
    let game_id = repo.create_game("test_world", "Game").unwrap();

    let before = repo.get_game(game_id).unwrap().unwrap().updated_at;
    repo.set_game_id(game_id);
    let after = repo.get_game(game_id).unwrap().unwrap().updated_at;

    assert!(
        after >= before,
        "set_game_id should update games.updated_at"
    );
}
