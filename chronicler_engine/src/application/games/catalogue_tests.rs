//! Unit tests for GameCatalogue.

use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::message_service::MessageService;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::test_support::TestDataBuilder;

fn seeded_catalogue() -> (GameCatalogue, Arc<Storage>, String, String) {
    let data = TestDataBuilder::default_test().build();
    let storage = Arc::new(Storage::new_in_memory());
    data.seed_into(&storage);
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let world_key = data.world_key();
    let persona_key = data.persona.key.clone();
    (
        GameCatalogue::new(Arc::clone(&storage), message_service),
        storage,
        world_key,
        persona_key,
    )
}

#[test]
fn create_game_returns_positive_id() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();

    let id = catalogue
        .create_game(&world_key, &persona_key)
        .expect("create_game should succeed");

    assert!(id > 0, "Game ID should be positive");
    let game = storage
        .get_game(id)
        .unwrap()
        .expect("game should be persisted");
    assert_eq!(game.world_key, world_key);
    assert_eq!(game.persona_key, persona_key);
    assert_eq!(
        storage.current_game_id(),
        id,
        "new game should become current"
    );
}

#[test]
fn create_game_errors_when_world_missing() {
    let (catalogue, _storage, _world_key, persona_key) = seeded_catalogue();

    let result = catalogue.create_game("no_such_world", &persona_key);

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("World not found")),
        "Expected world-not-found validation error, got {result:?}"
    );
}

#[test]
fn create_game_errors_when_persona_missing() {
    let (catalogue, _storage, world_key, _persona_key) = seeded_catalogue();

    let result = catalogue.create_game(&world_key, "no_such_persona");

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Persona not found")),
        "Expected persona-not-found validation error, got {result:?}"
    );
}

#[test]
fn create_game_generates_unique_names() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();

    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id3 = catalogue.create_game(&world_key, &persona_key).unwrap();

    let g1 = catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .find(|g| g.id == id1)
        .unwrap();
    let g2 = catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .find(|g| g.id == id2)
        .unwrap();
    let g3 = catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .find(|g| g.id == id3)
        .unwrap();

    assert_ne!(g1.name, g2.name, "game names should be unique");
    assert_ne!(g2.name, g3.name, "game names should be unique");
    assert_ne!(g1.name, g3.name, "game names should be unique");
}

#[test]
fn create_game_restores_current_game_on_persist_failure() {
    let data = TestDataBuilder::default_test().build();
    let raw_storage = Storage::new_in_memory();
    data.seed_into(&raw_storage);
    let (storage, handle) = raw_storage.with_test_failures();
    handle.set(
        "save_snapshot",
        TestOverride::internal("simulated initial snapshot failure"),
    );
    let storage = Arc::new(storage);
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let catalogue = GameCatalogue::new(Arc::clone(&storage), message_service);
    let original_current = storage.current_game_id();

    let result = catalogue.create_game(&data.world_key(), &data.persona.key);

    assert!(
        result.is_err(),
        "create_game should fail when initial state cannot be persisted"
    );
    assert_eq!(
        storage.current_game_id(),
        original_current,
        "current game should be restored after rollback"
    );
}

#[test]
fn switch_game_changes_current_game() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();
    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();

    catalogue
        .switch_game(id1)
        .expect("switch to first game should succeed");
    assert_eq!(catalogue.current_game_id(), id1);

    catalogue
        .switch_game(id2)
        .expect("switch to second game should succeed");
    assert_eq!(catalogue.current_game_id(), id2);
}

#[test]
fn switch_game_errors_when_game_missing() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();

    let result = catalogue.switch_game(9999);

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Game not found")),
        "Expected game-not-found validation error, got {result:?}"
    );
}

#[test]
fn delete_game_removes_non_active_game() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();
    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();
    catalogue.switch_game(id1).unwrap();

    catalogue
        .delete_game(id2)
        .expect("delete non-active game should succeed");

    assert!(
        storage.get_game(id2).unwrap().is_none(),
        "deleted game should not exist"
    );
}

#[test]
fn delete_game_errors_when_deleting_active_game() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();
    let id = catalogue.create_game(&world_key, &persona_key).unwrap();

    let result = catalogue.delete_game(id);

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Cannot delete the active game")),
        "Expected active-game deletion error, got {result:?}"
    );
}

#[test]
fn list_games_returns_all_games() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();
    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();

    let games = catalogue.list_games().expect("list_games should succeed");
    let ids: Vec<_> = games.iter().map(|g| g.id).collect();

    assert!(ids.contains(&id1), "list should contain first game");
    assert!(ids.contains(&id2), "list should contain second game");
}

#[test]
fn reset_replaces_current_game() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();
    catalogue.create_game(&world_key, &persona_key).unwrap();
    let pre_reset_current = catalogue.current_game_id();

    catalogue.reset().expect("reset should succeed");

    let post_reset_current = catalogue.current_game_id();
    assert_ne!(
        post_reset_current, pre_reset_current,
        "reset should create a new current game"
    );
    assert!(
        storage.get_game(pre_reset_current).unwrap().is_none(),
        "pre-reset game should be deleted"
    );
}

#[test]
fn current_game_id_matches_storage() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    assert_eq!(catalogue.current_game_id(), storage.current_game_id());
}
