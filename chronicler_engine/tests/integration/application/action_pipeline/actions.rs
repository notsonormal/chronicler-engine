//! Integration tests for the action pipeline: verifies that user actions are persisted to state, that narrations from the LLM are stored, and that error paths (room not found, LLM failure) are surfaced gracefully.

use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;
use chronicler_engine::test_support::{
    make_test_app_with_backends, make_test_app_with_separate_backends,
};

use crate::{fixtures::create_test_state, pipeline_helpers::latest_state};
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::application::action_pipeline::execute_action_impl;

#[test]
fn test_pipeline_executes_and_persists_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "look".to_string());

    let final_state = latest_state(&app);
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "Pipeline should generate and persist narration"
    );
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Status should not be generating after completion"
    );
}

#[test]
fn test_pipeline_persists_input_before_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "examine the room".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "examine the room".to_string());

    let final_state = latest_state(&app);
    let entries: Vec<_> = final_state.narrative.history().into_iter().collect();
    let input_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Input && e.text == "examine the room");
    let narration_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Narration);
    assert!(input_idx.is_some(), "Input should be persisted");
    assert!(narration_idx.is_some(), "Narration should be persisted");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before narration"
    );
}

#[test]
fn test_pipeline_handles_room_not_found() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.movement.current_room_id = "non_existent_room".to_string();
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "look".to_string());

    let final_state = latest_state(&app);
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should reset generating status when room not found"
    );
}

#[test]
fn test_pipeline_handles_llm_failure() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let app = make_test_app_with_separate_backends(
        state,
        || MockBackend::default().with_fail(),
        MockBackend::default,
    )
    .unwrap();

    execute_action_impl(&app, "look".to_string());

    let final_state = latest_state(&app);
    assert!(
        final_state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "Should set error status when LLM fails"
    );
}

#[test]
fn test_pipeline_clears_last_trigger() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.last_trigger = Some(StoredTriggerContext {
        trigger_name: "Old".to_string(),
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_repeat: false,
        trigger_narration_prompt: "The old trigger fires.".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "look".to_string());

    let final_state = latest_state(&app);
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared"
    );
}

#[test]
fn test_pipeline_phase_transitions() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    execute_action_impl(&app, "look".to_string());

    let guard = latest_state(&app);
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after successful completion"
    );
}

#[test]
fn test_pipeline_phase_stays_narrating_on_error() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let app = make_test_app_with_separate_backends(
        state,
        || MockBackend::default().with_fail(),
        MockBackend::default,
    )
    .unwrap();

    execute_action_impl(&app, "look".to_string());

    let guard = latest_state(&app);
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::Narrating,
        "Phase should remain Narrating after failed narration"
    );
}

#[test]
fn test_pipeline_empty_input() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    execute_action_impl(&app, String::new());

    let guard = latest_state(&app);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Empty input should complete generation: {:?}",
        guard.narrative.input_buffer.status
    );
    let has_narration = guard
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "Empty input should produce continuation narration"
    );
    let has_input = guard
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Input);
    assert!(!has_input, "Empty input should not add an Input message");
}
