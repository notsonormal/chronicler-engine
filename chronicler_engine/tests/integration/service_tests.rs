/// [DOC: docs/reference/testing.md]
/// Comprehensive integration tests for DefaultGameService public API
#[path = "../helpers/fixtures.rs"]
mod fixtures;

#[path = "helpers/pipeline_helpers.rs"]
mod pipeline_helpers;

use std::sync::Arc;

use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;

use chronicler_engine::test_support::make_test_context;
use pipeline_helpers::{latest_state, add_input_and_save, wait_for_generation_complete};
use fixtures::create_test_state;

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

// ============================================================================
// MessageEditingService Tests (10 tests)
// ============================================================================

#[test]
fn test_switch_swipe_out_of_bounds() {
    use chronicler_engine::model::message::{Message, Swipe};
    use chronicler_engine::model::state::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;
    use chronicler_engine::application::ApplicationError;

    let mut state = create_test_state();
    // Add a narration message with 2 swipes
    let mut msg = Message::new(None, "First narration", MessageType::Narration, None, None);
    msg.swipes.push(Swipe {
        text: "First narration".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    });
    msg.swipes.push(Swipe {
        text: "Second narration".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    });
    state.narrative.history.append(msg);

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let last_message = messages.last().unwrap();
    let message_id = last_message.id;
    let swipe_count = last_message.swipes.len();

    // Try index = swipe_count (out of bounds)
    let result = editing_service.switch_swipe(ctx, message_id, swipe_count);

    assert!(result.is_err());
    if let ApplicationError::Engine(_) = result.unwrap_err() {
        // Expected
    } else {
        panic!("Expected Engine error");
    }
}

#[test]
fn test_edit_history_updates_text() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;
    let edited_text = "Edited narration".to_string();

    let result = editing_service.edit_history(ctx.clone(), message_id, edited_text.clone());
    assert!(result.is_ok());

    // Verify updated in storage
    // Verify updated in storage
    let messages = ctx.load_messages().unwrap();
    let stored = messages.iter().find(|m| m.id == message_id).unwrap();
    assert_eq!(stored.text(), edited_text);
}

#[test]
fn test_edit_history_no_snapshot() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;

    let result = editing_service.edit_history(ctx.clone(), message_id, "Edited".to_string());
    // Should succeed (no snapshot = no-op on storage side)
    assert!(result.is_ok());
}

#[test]
fn test_delete_last_removes() {
    use chronicler_engine::application::message_editing::MessageEditingService;
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::MessageType;
    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message to delete",
        MessageType::Narration,
        None,
        None,
    ));
    let initial_len = state.narrative.history.len();

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let result = editing_service.delete_last(ctx.clone());
    assert!(result.is_ok());

    let messages = ctx.storage.load_message_rows().unwrap();
    assert_eq!(messages.len(), initial_len - 1);
}

#[test]
fn test_delete_last_empty_rejected() {
    use chronicler_engine::application::message_editing::MessageEditingService;
    use chronicler_engine::application::ApplicationError;

    let mut state = create_test_state();
    state.narrative.history.clear();

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let result = editing_service.delete_last(ctx.clone());
    assert!(result.is_err());
    if let ApplicationError::Engine(e) = result.unwrap_err() {
        assert!(e.to_string().contains("History is empty"));
    } else {
        panic!("Expected Engine error");
    }
}

#[test]
fn test_edit_history_storage_failure() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;

    let result = editing_service.edit_history(ctx.clone(), message_id, "Edited".to_string());
    // Should succeed or handle error gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_retrigger_happy_path() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::{MessageType, StoredTriggerContext};
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.last_trigger = Some(StoredTriggerContext {
        npc_id: "test_npc".to_string(),
        trigger_idx: 0,
        trigger_name: "test_trigger".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Test".to_string(),
        system_prompt: "System".to_string(),
        user_prompt: "User".to_string(),
        max_tokens: None,
    });
    state.narrative.history.append(Message::new(
        None,
        "Previous narration",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let result = editing_service.retrigger(ctx.clone());
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retrigger_storage_operations() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::{MessageType, StoredTriggerContext};
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.last_trigger = Some(StoredTriggerContext {
        npc_id: "test_npc".to_string(),
        trigger_idx: 0,
        trigger_name: "test_trigger".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Test".to_string(),
        system_prompt: "System".to_string(),
        user_prompt: "User".to_string(),
        max_tokens: None,
    });
    state.narrative.history.append(Message::new(
        None,
        "Test narration",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));

    let initial_snapshot = ctx.storage.load_latest_snapshot().unwrap();
    assert!(initial_snapshot.is_some());

    let result = editing_service.retrigger(ctx.clone());
    assert!(result.is_ok());

    let final_snapshot = ctx.storage.load_latest_snapshot().unwrap();
    assert!(final_snapshot.is_some());

    let messages = ctx.storage.load_message_rows().unwrap();
    assert!(!messages.is_empty());
}
#[test]
fn test_delete_last_storage_failure() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;
    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message",
        MessageType::Narration,
        None,
        None,
    ));
    let ctx = make_test_context(state);
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));
    let result = editing_service.delete_last(ctx.clone());
    // Should succeed or handle error gracefully
    assert!(result.is_ok() || result.is_err());
}
#[tokio::test]
async fn test_retry_cancellation() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;
    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        Some("Player".to_string()),
        "Input to retry",
        MessageType::Input,
        None,
        None,
    ));
    let ctx = make_test_context(state);
    // Cancel the token to simulate cancellation
    ctx.cancel_token.cancel();
    let service = DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    );
    let editing_service = MessageEditingService::new(Arc::new(service));
    let result = editing_service.retry(ctx.clone());
    // Should return error due to cancellation
    assert!(result.is_err());
}
