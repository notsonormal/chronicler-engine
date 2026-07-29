//! GameService integration tests
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;

use crate::sqlite_test_app_builder::SqliteTestAppBuilder;
use crate::application_ext::PipelineHelpers;

#[test]
fn test_with_storage_uses_external() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .mock_backend(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.execute_action("test".to_string());

    let guard = app.latest_state(&pg);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Service should complete action execution"
    );
}

#[test]
fn test_with_backends_no_disk_read() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.execute_action("test input".to_string());

    let guard = app.latest_state(&pg);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Explicit backends should complete execution"
    );
}

#[test]
fn test_execute_action_saves_narration() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();
    let initial_history_len = 0_usize;

    app.execute_action("test action".to_string());

    let final_state = app.latest_state(&pg);
    let final_history_len = final_state.narrative.history.len();

    assert!(
        final_history_len > initial_history_len,
        "History should grow after action execution (before={initial_history_len}, after={final_history_len})"
    );
}

#[test]
fn test_execute_action_empty_input() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.execute_action("".to_string());

    let guard = app.latest_state(&pg);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Empty input should not leave pipeline in generating state"
    );
}

#[test]
fn test_execute_action_clears_last_trigger() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.execute_action("first action".to_string());
    app.execute_action("second action".to_string());

    let guard = app.latest_state(&pg);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Trigger state should be cleared between executions"
    );
}

#[test]
fn test_execute_action_cancellation() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .separate_backends(
            || MockBackend::default().with_delay(100),
            MockBackend::default,
        )
        .build_with_state()
        .unwrap();

    app.cancel_token().cancel();

    app.execute_action("test".to_string());

    let guard = app.latest_state(&pg);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Cancelled execution should not remain in generating state"
    );
}

#[test]
fn test_retry_finds_anchor() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.add_input_and_save(&pg, "Test input");

    app.retry_last_response();

    let final_state = app.latest_state(&pg);
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Retry should complete without hanging"
    );
}

#[test]
fn test_retry_event_fallback() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.retry_last_response();

    let guard = app.latest_state(&pg);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Empty history retry should fail gracefully without hanging"
    );
}

#[test]
fn test_retry_empty_history() {
    let (app, _pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    app.retry_last_response();

    let messages = app.storage().load_message_rows().unwrap();
    assert!(
        messages.is_empty(),
        "Retry with empty history should not create spurious messages"
    );
}

#[test]
fn test_switch_swipe_out_of_bounds() {
    use chronicler_engine::application::ApplicationError;
    use chronicler_engine::domain::model::message::{Message, Swipe};

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

    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .message(msg)
        .build_with_state()
        .unwrap();

    let messages = pg.load_messages().unwrap();
    let last_message = messages.last().unwrap();
    let message_id = last_message.id;
    let swipe_count = last_message.swipes.len();

    let result = app.switch_swipe(message_id, swipe_count);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::Engine(_)));
}

#[test]
fn test_edit_history_updates_text() {
    use chronicler_engine::domain::model::message::Message;
    use chronicler_engine::domain::model::state::message_types::MessageType;

    let msg = Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    let (_app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .message(msg)
        .build_with_state()
        .unwrap();

    let messages = pg.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;
    let edited_text = "Edited narration".to_string();

    let result = pg.edit_history(message_id, edited_text.clone());
    assert!(result.is_ok());

    let messages = pg.load_messages().unwrap();
    let stored = messages.iter().find(|m| m.id == message_id).unwrap();
    assert_eq!(stored.text(), edited_text);
}

#[test]
fn test_edit_history_no_snapshot() {
    use chronicler_engine::domain::model::message::Message;
    use chronicler_engine::domain::model::state::message_types::MessageType;

    let msg = Message::new(None, "Test message", MessageType::Narration, None, None);
    let (_app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .message(msg)
        .build_with_state()
        .unwrap();

    let messages = pg.load_messages().unwrap();
    let message_id = messages.last().unwrap().id;

    let result = pg.edit_history(message_id, "Edited".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_delete_last_removes() {
    use chronicler_engine::domain::model::message::Message;
    use chronicler_engine::domain::model::state::message_types::MessageType;
    let msg = Message::new(
        None,
        "Test message to delete",
        MessageType::Narration,
        None,
        None,
    );
    let initial_len = 1_usize;
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .message(msg)
        .build_with_state()
        .unwrap();

    let result = pg.delete_last();
    assert!(result.is_ok());

    let messages = app.storage().load_message_rows().unwrap();
    assert_eq!(messages.len(), initial_len - 1);
}

#[test]
fn test_delete_last_empty_rejected() {
    use chronicler_engine::application::ApplicationError;

    let (_app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    let result = pg.delete_last();
    assert!(result.is_err());
    if let ApplicationError::Engine(e) = result.unwrap_err() {
        assert!(e.to_string().contains("History is empty"));
    } else {
        panic!("Expected Engine error");
    }
}

#[tokio::test]
async fn test_retrigger_happy_path() {
    use chronicler_engine::domain::model::message::Message;
    use chronicler_engine::domain::model::state::message_types::MessageType;
    use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;

    let msg = Message::new(
        None,
        "Previous narration",
        MessageType::Narration,
        None,
        None,
    );
    let (app, _pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .last_trigger(StoredTriggerContext {
            npc_id: "test_npc".to_string(),
            trigger_idx: 0,
            trigger_name: "test_trigger".to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: "Test".to_string(),
            system_prompt: "System".to_string(),
            user_prompt: "User".to_string(),
            max_tokens: None,
        })
        .message(msg)
        .build_with_state()
        .unwrap();

    let result = app.retrigger();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retrigger_storage_operations() {
    use chronicler_engine::domain::model::message::Message;
    use chronicler_engine::domain::model::state::message_types::MessageType;
    use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;

    let msg = Message::new(None, "Test narration", MessageType::Narration, None, None);
    let (app, _pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .last_trigger(StoredTriggerContext {
            npc_id: "test_npc".to_string(),
            trigger_idx: 0,
            trigger_name: "test_trigger".to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: "Test".to_string(),
            system_prompt: "System".to_string(),
            user_prompt: "User".to_string(),
            max_tokens: None,
        })
        .message(msg)
        .build_with_state()
        .unwrap();

    let initial_snapshot = app.storage().load_latest_snapshot().unwrap();
    assert!(initial_snapshot.is_some());

    let result = app.retrigger();
    assert!(result.is_ok());

    let final_snapshot = app.storage().load_latest_snapshot().unwrap();
    assert!(final_snapshot.is_some());

    let messages = app.storage().load_message_rows().unwrap();
    assert!(!messages.is_empty());
}

#[tokio::test]
async fn test_retry_cancellation() {
    use chronicler_engine::domain::model::message::Message;
    use chronicler_engine::domain::model::state::message_types::MessageType;
    let msg = Message::new(
        Some("Player".to_string()),
        "Input to retry",
        MessageType::Input,
        None,
        None,
    );
    let (app, _pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .message(msg)
        .build_with_state()
        .unwrap();
    app.cancel_token().cancel();
    let result = app.retry();
    assert!(result.is_err());
}

#[test]
fn test_continue_narration_fresh_game() {
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();

    let initial_history = app.latest_state(&pg).narrative.history.len();
    app.execute_action(String::new());

    let guard = app.latest_state(&pg);
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
    let (app, pg) = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_with_state()
        .unwrap();
    app.set_is_generating(true);

    app.execute_action(String::new());

    let final_state = app.latest_state(&pg);
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
        let (app, pg) = SqliteTestAppBuilder::default_test()
            .backends(MockBackend::default)
            .build_with_state()
            .unwrap();
        app.execute_action(whitespace.to_string());

        let final_state = app.latest_state(&pg);
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
