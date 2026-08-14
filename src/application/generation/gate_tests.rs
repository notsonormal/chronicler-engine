//! Unit tests for GenerationGate.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::AppState;
use crate::application::agents::registry::AgentRegistry;
use crate::application::generation::gate::GenerationGate;
use crate::application::message_service::MessageService;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::test_support::fixtures::{TestMap, TestWorld};
use crate::test_support::{make_test_pipeline_with_backends, make_test_recorder};

#[test]
fn test_heal_stale_resets_generating_status_when_no_active_slot() {
    let storage = Arc::new(Storage::new_in_memory());
    let _message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let gate = GenerationGate::new();
    let mut state = GameState::new("room_1".to_string());
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;

    gate.heal_stale(1, &mut state);

    assert!(
        !state.narrative.input_buffer.status.is_generating(),
        "stale Generating status should be reset to Idle"
    );
    assert_eq!(
        state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "phase should be reset to default"
    );
}

#[test]
fn test_heal_stale_leaves_generating_status_when_slot_active() {
    let storage = Arc::new(Storage::new_in_memory());
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let gate = GenerationGate::new();
    let mut state = GameState::new("room_1".to_string());
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;

    let (_game_id, _generation_id, result) = gate
        .try_claim(1, &mut state, message_service.as_ref())
        .expect("try_claim should succeed");
    assert!(matches!(
        result,
        crate::application::errors::ProcessActionResult::Started
    ));

    let mut fresh_state = GameState::new("room_1".to_string());
    fresh_state.narrative.input_buffer.status = GenerationStatus::Generating;
    fresh_state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    gate.heal_stale(1, &mut fresh_state);

    assert!(
        fresh_state.narrative.input_buffer.status.is_generating(),
        "active slot should keep Generating status intact"
    );
}

#[test]
fn test_heal_stale_is_noop_when_status_is_idle() {
    let gate = GenerationGate::new();
    let mut state = GameState::new("room_1".to_string());
    state.narrative.input_buffer.status = GenerationStatus::Idle;

    gate.heal_stale(1, &mut state);

    assert!(
        !state.narrative.input_buffer.status.is_generating(),
        "Idle status should remain unchanged"
    );
}

fn minimal_state() -> GameState {
    GameState::new("start")
}

#[test]
fn test_reset_generating_status_sets_idle() {
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();
    let mock: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let narrator_recorder = make_test_recorder(Arc::clone(&mock));
    let pipeline = make_test_pipeline_with_backends(
        Arc::clone(&storage),
        narrator_recorder,
        AgentRegistry::default(),
    );
    let wired = crate::test_support::build_test_wired_app(Arc::clone(&storage), pipeline)
        .expect("build_test_wired_app should succeed");
    let app = AppState::from_wired(wired);

    let game_id = app.game_catalogue.current_game_id();
    let _ = app
        .generation_gate
        .release_generation_slot_for_game(game_id);
    let result = app.pipeline.reset_persisted_status();
    assert!(result.is_ok());
    let (status, _) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

#[test]
fn test_boot_heal_resets_stale_generating_status() {
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();

    let mut state = minimal_state();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    let _ = storage.save_snapshot(&GameStateSnapshot::from_game_state(&state));

    let mock: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let narrator_recorder = make_test_recorder(Arc::clone(&mock));
    let pipeline = make_test_pipeline_with_backends(
        Arc::clone(&storage),
        narrator_recorder,
        AgentRegistry::default(),
    );
    let wired = crate::test_support::build_test_wired_app(storage, pipeline)
        .expect("build_test_wired_app should succeed");
    let app = AppState::from_wired(wired);

    let (status, phase) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
    assert_eq!(phase, GenerationPhase::default());
}
