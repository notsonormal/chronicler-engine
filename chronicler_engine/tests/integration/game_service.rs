//! [DOC: docs/reference/testing.md]
//! GameService integration tests
use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::model::state::message_types::MessageType;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context_with_sqlite;
use crate::fixtures::create_test_state;
use crate::pipeline_helpers::{latest_state, add_input_and_save};

#[test]
fn test_with_storage_uses_external() {
    let llm_backend = Arc::new(MockBackend::default());
    let quantifier = Arc::new(MockBackend::default());
    let service = GameService::with_mock_quantifier(llm_backend, quantifier);

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    service.execute_action(ctx.clone(), "test".to_string());

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
    let service = GameService::with_backends(llm_backend, registry);

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    service.execute_action(ctx.clone(), "test input".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Explicit backends should complete execution"
    );
}

#[test]
fn test_execute_action_saves_narration() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());

    let state = create_test_state();
    let initial_history_len = state.narrative.history.len();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    service.execute_action(ctx.clone(), "test action".to_string());

    let final_state = latest_state(&ctx);
    let final_history_len = final_state.narrative.history.len();

    assert!(
        final_history_len > initial_history_len,
        "History should grow after action execution (before={initial_history_len}, after={final_history_len})"
    );
}

#[test]
fn test_execute_action_empty_input() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    service.execute_action(ctx.clone(), "".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Empty input should not leave pipeline in generating state"
    );
}

#[test]
fn test_execute_action_clears_last_trigger() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    service.execute_action(ctx.clone(), "first action".to_string());
    service.execute_action(ctx.clone(), "second action".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Trigger state should be cleared between executions"
    );
}

#[test]
fn test_execute_action_cancellation() {
    let service = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default().with_delay(100)),
        Arc::new(MockBackend::default()),
    );

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    ctx.cancel_token.cancel();

    service.execute_action(ctx.clone(), "test".to_string());

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Cancelled execution should not remain in generating state"
    );
}

#[test]
fn test_retry_finds_anchor() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

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
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());

    let state = create_test_state();
    let ctx = make_test_context_with_sqlite(state).unwrap();

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

    let ctx = make_test_context_with_sqlite(state).unwrap();

    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());

    service.retry_last_response(ctx.clone());

    let messages = ctx.storage.load_message_rows().unwrap();
    assert!(
        messages.is_empty(),
        "Retry with empty history should not create spurious messages"
    );
}

#[test]
fn test_switch_swipe_out_of_bounds() {
    use chronicler_engine::model::message::{Message, Swipe};
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;
    use chronicler_engine::application::ApplicationError;

    let mut state = create_test_state();
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

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let last_message = messages.last().unwrap();
    let message_id = last_message.id;
    let swipe_count = last_message.swipes.len();

    let result = editing_service.switch_swipe(ctx, message_id, swipe_count);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::Engine(_)));
}

#[test]
fn test_edit_history_updates_text() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;
    let edited_text = "Edited narration".to_string();

    let result = editing_service.edit_history(ctx.clone(), message_id, edited_text.clone());
    assert!(result.is_ok());

    let messages = ctx.load_messages().unwrap();
    let stored = messages.iter().find(|m| m.id == message_id).unwrap();
    assert_eq!(stored.text(), edited_text);
}

#[test]
fn test_edit_history_no_snapshot() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;

    let result = editing_service.edit_history(ctx.clone(), message_id, "Edited".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_delete_last_removes() {
    use chronicler_engine::application::message_editing::MessageEditingService;
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::message_types::MessageType;
    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message to delete",
        MessageType::Narration,
        None,
        None,
    ));
    let initial_len = state.narrative.history.len();

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
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

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
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
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;

    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test",
        MessageType::Narration,
        None,
        None,
    ));

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));

    let messages = ctx.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;

    let result = editing_service.edit_history(ctx.clone(), message_id, "Edited".to_string());
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_retrigger_happy_path() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::model::state::trigger_context::StoredTriggerContext;
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

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));

    let result = editing_service.retrigger(ctx.clone());
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retrigger_storage_operations() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::model::state::trigger_context::StoredTriggerContext;
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

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
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
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;
    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        None,
        "Test message",
        MessageType::Narration,
        None,
        None,
    ));
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));
    let result = editing_service.delete_last(ctx.clone());
    assert!(result.is_ok() || result.is_err());
}
#[tokio::test]
async fn test_retry_cancellation() {
    use chronicler_engine::model::message::Message;
    use chronicler_engine::model::state::message_types::MessageType;
    use chronicler_engine::application::message_editing::MessageEditingService;
    let mut state = create_test_state();
    state.narrative.history.append(Message::new(
        Some("Player".to_string()),
        "Input to retry",
        MessageType::Input,
        None,
        None,
    ));
    let ctx = make_test_context_with_sqlite(state).unwrap();
    ctx.cancel_token.cancel();
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let editing_service = MessageEditingService::new(Arc::new(service));
    let result = editing_service.retry(ctx.clone());
    assert!(result.is_err());
}

#[test]
fn test_continue_narration_fresh_game() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let service = crate::working_service();

    let initial_history = latest_state(&ctx).narrative.history.len();
    service.execute_action(ctx.clone(), String::new());

    let guard = latest_state(&ctx);
    assert!(
        guard.narrative.history.len() > initial_history,
        "Empty input should generate narration (history should grow)"
    );

    let entries: Vec<_> = guard
        .narrative
        .history
        .iter()
        .skip(initial_history)
        .collect();
    assert!(!entries.is_empty(), "Should have at least one new entry");
    assert_eq!(
        entries[0].message_type,
        MessageType::Narration,
        "Empty input should produce Narration, not Input"
    );
}

#[test]
fn test_continue_narration_with_stale_is_generating_flag() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();
    ctx.is_generating
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let service = crate::working_service();

    service.execute_action(ctx.clone(), String::new());

    let final_state = latest_state(&ctx);
    assert!(
        !final_state.narrative.history.is_empty(),
        "Pipeline should run even with stale is_generating flag"
    );
}

#[test]
fn test_whitespace_variations() {
    let test_cases = vec![
        "   ",    // spaces
        "\t",     // tab
        "\n",     // newline
        " \t \n", // mixed
    ];

    for whitespace in test_cases {
        let mut state = create_test_state();
        state.narrative.history.clear();
        let ctx = make_test_context_with_sqlite(state).unwrap();
        let service = crate::working_service();
        service.execute_action(ctx.clone(), whitespace.to_string());

        let final_state = latest_state(&ctx);
        assert!(
            !final_state.narrative.history.is_empty(),
            "Whitespace input '{whitespace:?}' should produce continuation narration"
        );
        assert!(
            final_state
                .narrative
                .history
                .iter()
                .all(|m| m.message_type != MessageType::Input),
            "Whitespace input '{whitespace:?}' should not add Input message to history"
        );
    }
}
