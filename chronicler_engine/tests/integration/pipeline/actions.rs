use crate::{
    failing_service, fixtures::create_test_state, pipeline_helpers::latest_state, working_service,
};
use chronicler_engine::model::state::{
    GenerationPhase, GenerationStatus, MessageType, StoredTriggerContext,
};
use chronicler_engine::test_support::make_test_context_with_sqlite;

#[test]
fn test_pipeline_executes_and_persists_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

    let final_state = latest_state(&ctx);
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.execute_action(
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

    let final_state = latest_state(&ctx);
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should reset generating status when room not found"
    );
}

#[test]
fn test_pipeline_handles_llm_failure() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = failing_service();

    backend.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

    let final_state = latest_state(&ctx);
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

    let final_state = latest_state(&ctx);
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = failing_service();

    backend.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = failing_service();

    backend.execute_action(ctx.clone(), "".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        guard
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
            || !guard.narrative.input_buffer.status.is_generating(),
        "Empty input should complete without panic: {:?}",
        guard.narrative.input_buffer.status
    );
}
