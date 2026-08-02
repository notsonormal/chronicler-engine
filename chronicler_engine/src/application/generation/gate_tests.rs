//! Unit tests for GenerationGate.

use std::sync::Arc;

use crate::application::generation::gate::GenerationGate;
use crate::application::message_service::MessageService;
use crate::adapters::driven::storage::Storage;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};

#[test]
fn heal_stale_resets_generating_status_when_no_active_slot() {
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
fn heal_stale_leaves_generating_status_when_slot_active() {
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
fn heal_stale_is_noop_when_status_is_idle() {
    let gate = GenerationGate::new();
    let mut state = GameState::new("room_1".to_string());
    state.narrative.input_buffer.status = GenerationStatus::Idle;

    gate.heal_stale(1, &mut state);

    assert!(
        !state.narrative.input_buffer.status.is_generating(),
        "Idle status should remain unchanged"
    );
}
