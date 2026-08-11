//! Tests for `MessageService`.

use std::sync::Arc;

use crate::application::message_service::MessageService;
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::fixtures::{TestMap, TestPersona, TestWorld};

type Storage = crate::adapters::driven::storage::Storage;

fn make_service() -> (MessageService, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let service = MessageService::new(Arc::clone(&storage));
    (service, storage)
}

fn make_service_with_game() -> (MessageService, Arc<Storage>, u64) {
    let (service, storage) = make_service();
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    let world_catalogue = WorldCatalogue::new(Arc::clone(&storage));
    world_catalogue.create_world(world, map).unwrap();

    let persona = TestPersona::standard();
    storage.seed_persona(&persona.key, &persona).unwrap();

    let game_id = storage
        .create_game("Test World", "test", "hero", "Hero", "Test World 1")
        .unwrap();
    storage.set_game_id(game_id);

    (service, storage, game_id)
}

#[test]
fn test_load_expecting_valid_state_errors_when_no_snapshot() {
    let (service, _) = make_service();
    assert!(service.load_expecting_valid_state().is_err());
}

#[test]
fn test_load_or_fresh_returns_fresh_state_when_no_snapshot() {
    let (service, _) = make_service();
    let state = service.load_or_fresh();
    assert!(state.narrative.history().is_empty());
}

#[test]
fn test_save_state_and_load_roundtrip() {
    let (service, _) = make_service();
    let state = GameState::new("start");
    service.save_state(&state).unwrap();

    let loaded = service.load_expecting_valid_state().unwrap();
    assert_eq!(loaded.movement.current_room_id, "start");
}

#[test]
fn test_build_fresh_initial_state_uses_starting_room() {
    let (service, _, _game_id) = make_service_with_game();
    let state = service.build_fresh_initial_state().unwrap();
    assert_eq!(state.movement.current_room_id, "start");
}

#[test]
fn test_find_retry_anchor_empty_returns_none() {
    let (service, _) = make_service();
    assert!(service.find_retry_anchor(&[]).is_none());
}

#[test]
fn test_switch_swipe_rejects_concurrent_generation() {
    let (service, _, _game_id) = make_service_with_game();
    let err = service.switch_swipe(true, 1, 0).unwrap_err();
    assert!(matches!(
        err,
        crate::application::errors::ApplicationError::ConcurrentGeneration
    ));
}

#[test]
fn test_delete_last_removes_message_and_snapshot() {
    let (service, _storage, _game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), None, MessageType::Narration);
    service.save_message_and_snapshot(&mut state).unwrap();

    let before = service.load_messages().unwrap();
    assert_eq!(before.len(), 1);

    service.delete_last().unwrap();

    let after = service.load_messages().unwrap();
    assert!(after.is_empty());
}

#[test]
fn test_edit_history_updates_text_and_snapshot() {
    let (service, _storage, _game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("old".to_string(), None, MessageType::Narration);
    service.save_message_and_snapshot(&mut state).unwrap();

    let messages = service.load_messages().unwrap();
    let id = messages[0].id;

    service.edit_history(id, "new".to_string()).unwrap();

    let after = service.load_messages().unwrap();
    assert_eq!(after[0].text(), "new");
}

#[test]
fn test_save_message_and_snapshot_assigns_snapshot_id_to_message() {
    let (service, _storage, _game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), None, MessageType::Narration);
    assert!(
        state
            .narrative
            .history
            .last()
            .unwrap()
            .snapshot_id()
            .is_none()
    );

    service.save_message_and_snapshot(&mut state).unwrap();

    assert!(
        state
            .narrative
            .history
            .last()
            .unwrap()
            .snapshot_id()
            .is_some()
    );
    assert!(state.narrative.history.last().unwrap().id > 0);
}
