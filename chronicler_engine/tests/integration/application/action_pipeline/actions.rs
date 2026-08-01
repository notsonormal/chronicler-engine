//! Integration tests for the action pipeline: verifies that user actions are persisted to state, that narrations from the LLM are stored, and that error paths (room not found, LLM failure) are surfaced gracefully.

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;
use chronicler_engine::test_support::TestDataBuilder;

use crate::sqlite_test_app_builder::SqliteTestAppBuilder;
use crate::application_ext::PipelineHelpers;

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.1
#[test]
fn test_pipeline_executes_and_persists_narration() {
    let app = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let final_state = app.latest_state();
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

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.2
#[test]
fn test_pipeline_persists_input_before_narration() {
    let msg = Message::new(
        Some("Player".to_string()),
        "examine the room",
        MessageType::Input,
        None,
        None,
    );
    let app = SqliteTestAppBuilder::default_test()
        .message(msg)
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("examine the room".to_string());

    let final_state = app.latest_state();
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

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 2.1
#[test]
fn test_pipeline_handles_room_not_found() {
    let data = TestDataBuilder::default_test().build();
    let app = SqliteTestAppBuilder::with_data(data)
        .backends(MockBackend::default)
        .state_mut(|state| {
            state.movement.current_room_id = "non_existent_room".to_string();
        })
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let final_state = app.latest_state();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should reset generating status when room not found"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 2.2
#[test]
fn test_pipeline_handles_llm_failure() {
    let app = SqliteTestAppBuilder::default_test()
        .separate_backends(|| MockBackend::default().with_fail(), MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let final_state = app.latest_state();
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

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 3.1
#[test]
fn test_pipeline_clears_last_trigger() {
    let app = SqliteTestAppBuilder::default_test()
        .last_trigger(StoredTriggerContext {
            trigger_name: "Old".to_string(),
            npc_id: "npc1".to_string(),
            trigger_idx: 0,
            trigger_repeat: false,
            trigger_narration_prompt: "The old trigger fires.".to_string(),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        })
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let final_state = app.latest_state();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 3.4
#[test]
fn test_pipeline_phase_transitions() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Idle, GenerationPhase::default())
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let guard = app.latest_state();
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after successful completion"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 3.4
#[test]
fn test_pipeline_phase_stays_narrating_on_error() {
    let app = SqliteTestAppBuilder::default_test()
        .generation_status(GenerationStatus::Idle, GenerationPhase::default())
        .separate_backends(|| MockBackend::default().with_fail(), MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action("look".to_string());

    let guard = app.latest_state();
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::Narrating,
        "Phase should remain Narrating after failed narration"
    );
}

// [chronicler_engine/docs/specs/action_pipeline.md] SCENARIO: 1.5
#[test]
fn test_pipeline_empty_input() {
    let app = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.pipeline.execute_action(String::new());

    let guard = app.latest_state();
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
