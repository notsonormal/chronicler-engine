//! Tests for `MessageService`.

use std::sync::Arc;

use crate::application::message_service::MessageService;
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::game::GameId;
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::narrative_state::ArrivingLocationWrite;
use crate::test_support::fixtures::{TestMap, TestPersona, TestWorld};

type Storage = crate::adapters::driven::storage::Storage;

fn make_service() -> (MessageService, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let service = MessageService::new(Arc::clone(&storage));
    (service, storage)
}

fn make_service_with_game() -> (MessageService, Arc<Storage>, GameId) {
    let (service, storage) = make_service();
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    let world_catalogue = WorldCatalogue::new(Arc::clone(&storage));
    world_catalogue.create_world(world, map).unwrap();

    let persona = TestPersona::standard();
    storage.seed_persona(&persona.key, &persona).unwrap();

    let game_id = GameId(
        storage
            .create_game("Test World", "test", "hero", "Hero", "Test World 1")
            .unwrap(),
    );

    (service, storage, game_id)
}

#[test]
fn test_load_expecting_valid_state_errors_when_no_snapshot() {
    let (service, _) = make_service();
    assert!(service.load_expecting_valid_state(GameId(0)).is_err());
}

#[test]
fn test_load_or_fresh_returns_fresh_state_when_no_snapshot() {
    let (service, _) = make_service();
    let state = service.load_or_fresh(GameId(0));
    assert!(state.narrative.history().is_empty());
}

#[test]
fn test_save_state_and_load_roundtrip() {
    let (service, _) = make_service();
    let state = GameState::new("start");
    service.save_state(GameId(0), &state).unwrap();

    let loaded = service.load_expecting_valid_state(GameId(0)).unwrap();
    assert_eq!(loaded.swipe.movement.current_room_id, "start");
}

#[test]
fn test_build_fresh_initial_state_uses_starting_room() {
    let (service, _, game_id) = make_service_with_game();
    let state = service.build_fresh_initial_state(game_id).unwrap();
    assert_eq!(state.swipe.movement.current_room_id, "start");
}

#[test]
fn test_find_retry_anchor_empty_returns_none() {
    let (service, _) = make_service();
    assert!(service.find_retry_anchor(&[]).is_none());
}

#[test]
fn test_find_retry_anchor_guided_narration_after_input_anchors_the_guided_narration() {
    let (service, _) = make_service();
    let mut state = GameState::new("start");
    state.add_message("the older player input".to_string(), MessageType::Input);
    state.add_message("a normal narration".to_string(), MessageType::Narration);
    state.add_message_with_inputs(
        "a guided narration".to_string(),
        MessageType::Narration,
        false,
        Some("make it ominous".to_string()),
        None,
    );
    let messages = state.narrative.history.as_slice().to_vec();

    // The older Input's snapshot predates its narration, so it cannot anchor.
    let (idx, anchor) = service
        .find_retry_anchor_msg(&messages)
        .expect("anchor for a guided turn after an input");
    assert_eq!(idx, 2, "anchor must be the guided Narration");
    assert_eq!(anchor.text(), "a guided narration");
}

#[test]
fn test_find_retry_anchor_normal_turn_anchors_the_input() {
    let (service, _) = make_service();
    let mut state = GameState::new("start");
    state.add_message("the player input".to_string(), MessageType::Input);
    state.add_message("a narration".to_string(), MessageType::Narration);
    let messages = state.narrative.history.as_slice().to_vec();

    let (idx, anchor) = service
        .find_retry_anchor_msg(&messages)
        .expect("anchor for a normal turn");
    assert_eq!(idx, 0, "anchor must be the Input whose narration follows");
    assert_eq!(anchor.text(), "the player input");
}

#[test]
fn test_find_retry_anchor_trailing_system_notice_anchors_the_input() {
    let (service, _) = make_service();
    let mut state = GameState::new("start");
    state.add_message("the player input".to_string(), MessageType::Input);
    state.add_message("a narration".to_string(), MessageType::Narration);
    state.add_message(
        "[System] a trailing notice".to_string(),
        MessageType::System,
    );
    let messages = state.narrative.history.as_slice().to_vec();

    let (idx, anchor) = service
        .find_retry_anchor_msg(&messages)
        .expect("anchor for a normal turn with a trailing System notice");
    assert_eq!(idx, 0, "the notice must not decide the anchor");
    assert_eq!(anchor.text(), "the player input");
}

#[test]
fn test_find_retry_anchor_trailing_system_notice_after_an_event_anchors_the_main_narration() {
    let (service, _) = make_service();
    let mut state = GameState::new("start");
    state.add_message("the player input".to_string(), MessageType::Input);
    state.add_message("the main narration".to_string(), MessageType::Narration);
    state.add_message("the event narration".to_string(), MessageType::Narration);
    state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    state.add_message(
        "[System] a trailing notice".to_string(),
        MessageType::System,
    );
    let messages = state.narrative.history.as_slice().to_vec();

    let (idx, anchor) = service
        .find_retry_anchor_msg(&messages)
        .expect("anchor for an event continuation with a trailing System notice");
    assert_eq!(
        idx, 1,
        "the event continuation re-appends onto the main narration"
    );
    assert_eq!(anchor.text(), "the main narration");
}

#[test]
fn test_find_retry_anchor_guide_only_turn_anchors_the_guided_narration() {
    let (service, _) = make_service();
    let mut state = GameState::new("start");
    state.add_message_with_inputs(
        "a guided narration".to_string(),
        MessageType::Narration,
        false,
        Some("make it ominous".to_string()),
        None,
    );
    let messages = state.narrative.history.as_slice().to_vec();

    let (idx, anchor) = service
        .find_retry_anchor_msg(&messages)
        .expect("anchor for a guide-only turn");
    assert_eq!(idx, 0, "anchor must be the guided Narration itself");
    assert_eq!(anchor.text(), "a guided narration");
}

#[test]
fn test_switch_swipe_rejects_concurrent_generation() {
    let (service, _, game_id) = make_service_with_game();
    let err = service.switch_swipe(game_id, true, 1, 0).unwrap_err();
    assert!(matches!(
        err,
        crate::application::errors::ApplicationError::ConcurrentGeneration
    ));
}

fn swipe_with_snapshot(text: &str, snapshot_id: u64) -> Swipe {
    Swipe {
        text: text.to_string(),
        snapshot_id: Some(snapshot_id),
        location_header: None,
        event_header: None,
        impersonated: false,
        guide_text: None,
        impersonate_direction: None,
    }
}

#[test]
fn test_switch_swipe_restores_a_settled_snapshot() {
    let (service, storage, game_id) = make_service_with_game();

    let mut generating = GameState::new("start");
    generating.narrative.input_buffer.status = GenerationStatus::Generating;
    generating.narrative.input_buffer.phase = GenerationPhase::Narrating;
    generating.narrative.current_options = vec!["a stale option".to_string()];
    let snapshot_id = service.save_state(game_id, &generating).unwrap();

    let msg = Message::new("narration", MessageType::Narration, None, None);
    let id = storage.insert_message(game_id, &msg).unwrap();
    storage
        .insert_swipe(id, &swipe_with_snapshot("first", snapshot_id), 0)
        .unwrap();
    storage
        .insert_swipe(id, &swipe_with_snapshot("second", snapshot_id), 1)
        .unwrap();
    storage.update_active_swipe(game_id, id, 1).unwrap();

    service.switch_swipe(game_id, false, id, 0).unwrap();

    let restored = storage.load_latest_snapshot(game_id).unwrap().unwrap();
    assert_eq!(
        restored.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "a restored swipe must leave the game idle, not stuck generating"
    );
    assert_eq!(
        restored.narrative.input_buffer.phase,
        GenerationPhase::default()
    );
    assert!(
        restored.narrative.current_options.is_empty(),
        "a restored swipe must not offer another scene's option set"
    );
}

#[test]
fn test_delete_last_removes_message_and_snapshot() {
    let (service, _storage, game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), MessageType::Narration);
    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

    let before = service.load_messages(game_id).unwrap();
    assert_eq!(before.len(), 1);

    service.delete_last(game_id).unwrap();

    let after = service.load_messages(game_id).unwrap();
    assert!(after.is_empty());
}

#[test]
fn test_edit_history_updates_text_and_snapshot() {
    let (service, _storage, game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("old".to_string(), MessageType::Narration);
    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

    let messages = service.load_messages(game_id).unwrap();
    let id = messages[0].id;

    service
        .edit_history(game_id, id, "new".to_string())
        .unwrap();

    let after = service.load_messages(game_id).unwrap();
    assert_eq!(after[0].text(), "new");
}

#[test]
fn test_save_message_and_snapshot_assigns_snapshot_id_to_message() {
    let (service, _storage, game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), MessageType::Narration);
    assert!(
        state
            .narrative
            .history
            .last()
            .unwrap()
            .snapshot_id()
            .is_none()
    );

    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

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

#[test]
fn test_save_message_and_snapshot_persists_a_snapshot() {
    let (service, storage, game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), MessageType::Narration);
    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

    assert!(
        storage.load_latest_snapshot(game_id).unwrap().is_some(),
        "saving a message must persist the snapshot the restores read"
    );
}

#[test]
fn test_save_message_and_snapshot_writes_the_arriving_room_onto_the_saved_swipe() {
    let (service, _storage, game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), MessageType::Narration);
    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

    let narration = state.narrative.history.last().unwrap();
    state.narrative.arriving_location_write = Some(ArrivingLocationWrite {
        message_id: narration.id,
        swipe_index: narration.active_swipe_index,
        room_name: "Room start".to_string(),
    });
    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

    let stored = service.load_messages(game_id).unwrap();
    assert_eq!(
        stored[0].location_header(),
        Some("Room start"),
        "the write lands on the Swipe row the Narration was saved in"
    );
    assert_eq!(
        state.narrative.arriving_location_write, None,
        "a written room must not be applied twice"
    );
}

#[test]
fn test_a_failed_arriving_room_write_stays_for_the_next_save() {
    let (service, storage, game_id) = make_service_with_game();

    let mut state = GameState::new("start");
    state.add_message("hello".to_string(), MessageType::Narration);
    service
        .save_message_and_snapshot(game_id, &mut state)
        .unwrap();

    let narration = state.narrative.history.last().unwrap();
    let arriving = ArrivingLocationWrite {
        message_id: narration.id,
        swipe_index: narration.active_swipe_index,
        room_name: "Room start".to_string(),
    };
    state.narrative.arriving_location_write = Some(arriving.clone());
    storage.add_failure(
        "update_swipe_location_header",
        crate::adapters::driven::storage::TestOverride::internal("write failed"),
    );

    assert!(
        service
            .save_message_and_snapshot(game_id, &mut state)
            .is_err()
    );
    assert_eq!(
        state.narrative.arriving_location_write,
        Some(arriving),
        "the failed write must stay queued"
    );
}
