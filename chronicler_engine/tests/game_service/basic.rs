use std::sync::Arc;

use chronicler_engine::engine::game_service::{DefaultGameService, GameService};
use chronicler_engine::model::state::LogType;
use chronicler_engine::narrative::agents::quantifier::MockQuantifierBackend;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context;

use crate::failing_service;
use crate::test_data::create_test_state;

fn run_action(
    state: chronicler_engine::model::state::GameState,
    command: &str,
    service: &DefaultGameService,
) -> chronicler_engine::model::state::GameState {
    let mut state = state;
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    service.execute_action(ctx.clone(), command.to_string(), "Player".to_string());
    crate::latest_state(&ctx)
}

#[test]
fn test_execute_look_action() {
    let guard = run_action(create_test_state(), "look", &DefaultGameService::new());
    let has_narration = guard
        .narrative
        .history
        .iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(has_narration, "Look should add narration");
}

#[test]
fn test_execute_talk_action() {
    let guard = run_action(
        create_test_state(),
        "talk to innkeeper",
        &DefaultGameService::new(),
    );
    let has_system = guard
        .narrative
        .history
        .iter()
        .any(|e| e.log_type == LogType::System && e.text.contains("You talk to"));
    assert!(has_system, "Talk should add system log");
}

#[test]
fn test_execute_inventory_action() {
    let guard = run_action(create_test_state(), "inventory", &DefaultGameService::new());
    let has_system = guard
        .narrative
        .history
        .iter()
        .any(|e| e.log_type == LogType::System && e.text.contains("inventory"));
    assert!(has_system, "Inventory should add system log");
}

#[test]
fn test_execute_quit_action() {
    let guard = run_action(create_test_state(), "quit", &DefaultGameService::new());
    let has_goodbye = guard
        .narrative
        .history
        .iter()
        .any(|e| e.log_type == LogType::System && e.text.contains("Goodbye"));
    assert!(has_goodbye, "Quit should add Goodbye log");
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Quit should reset is_generating"
    );
}

#[test]
fn test_retry_with_no_history() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    let service = DefaultGameService::new();

    // Should not panic with empty history
    service.retry_last_response(ctx.clone());

    // State should be unchanged
    let guard = crate::latest_state(&ctx);
    assert!(guard.narrative.history.is_empty());
}

#[test]
fn test_execute_look_room_not_found() {
    let mut state = create_test_state();
    state.movement.current_room_id = "non_existent_room".to_string();
    let guard = run_action(state, "look", &DefaultGameService::new());
    assert!(
        !guard.narrative.generation.status.is_generating(),
        "Look should reset is_generating even when room not found"
    );
}

#[test]
fn test_execute_talk_no_message() {
    let guard = run_action(
        create_test_state(),
        "talk to innkeeper",
        &DefaultGameService::new(),
    );
    let has_talk = guard
        .narrative
        .history
        .iter()
        .any(|e| e.log_type == LogType::System && e.text.contains("You talk to innkeeper:"));
    assert!(has_talk, "Talk without message should add system log");
}

#[test]
fn test_execute_action_empty_command() {
    let guard = run_action(create_test_state(), "", &failing_service());
    // Failing mock backend causes FreeAction to fail and set Error status
    assert!(
        guard.narrative.generation.status.error_message().is_some(),
        "Empty command should result in error status: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_execute_action_unknown_command() {
    let guard = run_action(create_test_state(), "xyz123", &failing_service());
    // Failing mock backend causes FreeAction to fail and set Error status
    assert!(
        guard.narrative.generation.status.error_message().is_some(),
        "Unknown command should result in error status: {:?}",
        guard.narrative.generation.status
    );
}

#[test]
fn test_default_game_service_default() {
    // Default should be constructible (delegates to new())
    let _service: DefaultGameService = Default::default();
}

#[test]
fn test_default_game_service_with_backends() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    // Should be usable without panicking
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    service.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());
    let guard = crate::latest_state(&ctx);
    assert!(!guard.narrative.generation.status.is_generating());
}

#[test]
fn test_default_game_service_with_mock_quantifier() {
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockQuantifierBackend::default()),
    );
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    service.execute_action(ctx.clone(), "look".to_string(), "Player".to_string());
    let guard = crate::latest_state(&ctx);
    assert!(!guard.narrative.generation.status.is_generating());
}
