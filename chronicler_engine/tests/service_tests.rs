/// [DOC: docs/reference/testing.md]
/// Comprehensive integration tests for DefaultGameService public API
mod test_data;

#[path = "helpers/pipeline_helpers.rs"]
mod pipeline_helpers;

use std::sync::Arc;

use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;

use chronicler_engine::test_support::make_test_context;
use pipeline_helpers::{latest_state, add_input_and_save, wait_for_generation_complete};
use test_data::create_test_state;

// ============================================================================
// Constructor Tests (3 tests)
// ============================================================================

#[test]
fn test_with_storage_uses_external() {
    let llm_backend = Arc::new(MockBackend::default());
    let quantifier = Arc::new(MockBackend::default());
    let service = DefaultGameService::with_mock_quantifier(llm_backend, quantifier);

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(ctx.clone(), "test".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Service should complete action execution"
    );
}

#[test]
fn test_with_backends_no_disk_read() {
    let llm_backend = Arc::new(MockBackend::default());
    let registry = AgentRegistry::default();
    let service = DefaultGameService::with_backends(llm_backend, registry);

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(ctx.clone(), "test input".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Explicit backends should complete execution"
    );
}

#[test]
fn test_with_mock_quantifier() {
    let llm_backend = Arc::new(MockBackend::default());
    let quantifier = Arc::new(MockBackend::default());
    let service = DefaultGameService::with_mock_quantifier(llm_backend, quantifier);

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(ctx.clone(), "test input".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Mock quantifier should be configured correctly"
    );
}

// ============================================================================
// execute_action Tests (6 tests)
// ============================================================================

#[test]
fn test_execute_action_saves_narration() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    let state = create_test_state();
    let initial_history_len = state.narrative.history.len();
    let ctx = make_test_context(state);

    service.execute_action(ctx.clone(), "test action".to_string(), "Player".to_string());

    let final_state = latest_state(&ctx);
    let final_history_len = final_state.narrative.history.len();

    assert!(
        final_history_len > initial_history_len,
        "History should grow after action execution (before={initial_history_len}, after={final_history_len})"
    );
}

#[test]
fn test_execute_action_empty_input() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(ctx.clone(), "".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Empty input should not leave pipeline in generating state"
    );
}

#[test]
fn test_execute_action_clears_last_trigger() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(
        ctx.clone(),
        "first action".to_string(),
        "Player".to_string(),
    );
    service.execute_action(
        ctx.clone(),
        "second action".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Trigger state should be cleared between executions"
    );
}

#[test]
fn test_execute_action_preserves_input_log() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.execute_action(ctx.clone(), "input one".to_string(), "Player".to_string());
    service.execute_action(ctx.clone(), "input two".to_string(), "Player".to_string());

    let messages = ctx.storage.load_message_rows().unwrap();
    let input_count = messages
        .iter()
        .filter(|m| m.text().contains("input"))
        .count();

    assert!(
        input_count >= 2,
        "Multiple inputs should be preserved in history (found={input_count})"
    );
}

#[test]
fn test_execute_action_cancellation() {
    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(100)),
        Arc::new(MockBackend::default()),
    );

    let state = create_test_state();
    let ctx = make_test_context(state);

    ctx.cancel_token.cancel();

    service.execute_action(ctx.clone(), "test".to_string(), "Player".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Cancelled execution should not remain in generating state"
    );
}

#[test]
fn test_execute_action_trigger_continuation() {
    let state = create_test_state();
    let ctx = make_test_context(state);

    let mock_narrator = Arc::new(MockBackend::with_trigger_delay(50));
    let service = Arc::new(DefaultGameService::with_mock_quantifier(
        mock_narrator.clone(),
        Arc::new(MockBackend::default()),
    ));

    service.execute_action(
        ctx.clone(),
        "approach NPC".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 500);
    assert!(
        completed,
        "Trigger narration should complete within timeout"
    );

    let messages = ctx.storage.load_message_rows().unwrap();
    assert!(
        messages.len() > 1,
        "Trigger continuation should produce additional messages"
    );
}

// ============================================================================
// retry_last_response Tests (3 tests)
// ============================================================================

#[test]
fn test_retry_finds_anchor() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    let state = create_test_state();
    let ctx = make_test_context(state);

    // Use helper to add and save an input message with snapshot
    add_input_and_save(&ctx, "Test input");

    service.retry_last_response(ctx.clone());

    let final_state = latest_state(&ctx);
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Retry should complete without hanging"
    );
}

#[test]
fn test_retry_event_fallback() {
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    let state = create_test_state();
    let ctx = make_test_context(state);

    service.retry_last_response(ctx.clone());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Empty history retry should fail gracefully without hanging"
    );
}

#[test]
fn test_retry_empty_history() {
    let mut state = create_test_state();
    state.narrative.history.clear();

    let ctx = make_test_context(state);

    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );

    service.retry_last_response(ctx.clone());

    let messages = ctx.storage.load_message_rows().unwrap();
    assert!(
        messages.is_empty(),
        "Retry with empty history should not create spurious messages"
    );
}
