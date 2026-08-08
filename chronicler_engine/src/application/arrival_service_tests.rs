//! Unit tests for `ArrivalTaskContext`.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::application::arrival_service::ArrivalTaskContext;
use crate::application::message_service::MessageService;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::character::NpcCard;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{
    default_test_preset_storage, make_test_recorder_with_storage, seed_test_world_into_storage,
    TestDataBuilder,
};

fn make_test_arrival_task(
    storage: Arc<Storage>,
    room_id: &str,
) -> (ArrivalTaskContext, Arc<MessageService>) {
    let arrival_preset = default_test_preset_storage()
        .get_preset("system_default")
        .ok()
        .flatten()
        .expect("system_default preset should exist");

    let llm: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let recorder = make_test_recorder_with_storage(Arc::clone(&llm), Arc::clone(&storage));
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));

    let task_ctx = ArrivalTaskContext::new_for_test(
        Arc::clone(&message_service),
        Arc::clone(&storage),
        room_id.to_string(),
        Vec::<NpcCard>::new(),
        Vec::<NpcCard>::new(),
        Some(arrival_preset),
        "short".to_string(),
        1024,
        None,
        recorder,
    );
    (task_ctx, message_service)
}

#[test]
fn test_run_produces_and_persists_narration() {
    let data = TestDataBuilder::default_test().build();
    let storage = Arc::new(Storage::new_in_memory());
    data.seed_into(&storage);
    storage
        .save_snapshot(&GameStateSnapshot::from_game_state(&GameState::new(
            "room_1",
        )))
        .expect("test setup: save initial snapshot");

    let (task_ctx, message_service) = make_test_arrival_task(Arc::clone(&storage), "room_1");
    task_ctx.run_sync();

    let state = message_service.load_or_fresh();
    let narrations: Vec<_> = state
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.len() >= 2,
        "arrival run should persist scenario inject + arrival narrations (got {})",
        narrations.len()
    );
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "status should return to Idle after successful arrival narration"
    );

    let messages = storage.load_messages_with_swipes().unwrap();
    assert!(
        messages
            .iter()
            .any(|m| m.message_type == MessageType::Narration),
        "narration should be persisted to messages table"
    );

    let llm_messages = storage.list_latest_llm_messages(50).unwrap();
    assert!(
        llm_messages.iter().any(|m| m.agent_name == "narrator"),
        "narrator row should be persisted to llm_messages table"
    );
}

#[test]
fn test_run_falls_back_to_fresh_state_on_load_failure() {
    let state = GameState::new("room1");

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing_storage = Arc::new(failing_storage);
    seed_test_world_into_storage(&failing_storage, &state);
    handle.set(
        "load_latest_snapshot",
        TestOverride::internal("simulated load_latest_snapshot failure"),
    );

    let (task_ctx, message_service) = make_test_arrival_task(Arc::clone(&failing_storage), "room1");
    task_ctx.run_sync();

    handle.clear("load_latest_snapshot");

    let state = message_service.load_or_fresh();
    let narrations: Vec<_> = state
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        !narrations.is_empty(),
        "arrival run should fall back to fresh state and persist at least one Narration"
    );
}

#[test]
fn test_run_returns_early_without_narration_on_world_fetch_failure() {
    let state = GameState::new("room1");

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing_storage = Arc::new(failing_storage);
    seed_test_world_into_storage(&failing_storage, &state);
    failing_storage
        .save_snapshot(&GameStateSnapshot::from_game_state(&state))
        .expect("test setup: save initial snapshot");
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );

    let (task_ctx, message_service) = make_test_arrival_task(Arc::clone(&failing_storage), "room1");
    task_ctx.run_sync();

    handle.clear("get_world");

    let state = message_service.load_or_fresh();
    let narrations: Vec<_> = state
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.is_empty(),
        "arrival run must not add narration when world fetch fails, got {} narrations",
        narrations.len(),
    );
}
