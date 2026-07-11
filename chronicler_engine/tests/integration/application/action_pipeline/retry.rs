//! Integration tests for action retry behaviour: re-running the pipeline against the last user input, no-op on empty history, recovery after a previous LLM failure, and the missing-snapshot error path.

use std::sync::Arc;

use crate::{
    fixtures::create_test_state,
    pipeline_helpers::{
        create_test_state_with_trigger_npc, latest_state, wait_for_generation_complete,
    },
    sqlite_test_app_builder::SqliteTestAppBuilder,
};
use chronicler_engine::test_support::TestData;
use chronicler_engine::application::GameService;
use chronicler_engine::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;
use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::test_support::make_test_app_without_snapshot;
use chronicler_engine::TestAppBuilder;

fn trigger_data() -> TestData {
    TestData {
        world: Arc::new(crate::fixtures::create_test_world()),
        map: Arc::new(crate::fixtures::create_test_map()),
        persona: Arc::new(crate::fixtures::create_test_player()),
        npcs: vec![
            chronicler_engine::test_support::TestNpc::with_times_met_trigger(
                "shopkeeper",
                "Shopkeeper Sarah",
                chronicler_engine::domain::model::trigger::ComparisonOperator::Eq,
                0,
            ),
        ],
        room_npcs: vec!["shopkeeper".to_string()],
    }
}

#[test]
fn test_retry_finds_last_input_and_runs_pipeline() {
    let msg = Message::new(
        Some("Player".to_string()),
        "look around",
        MessageType::Input,
        None,
        None,
    );
    let app = SqliteTestAppBuilder::default_test()
        .message(msg)
        .backends(MockBackend::default)
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::execute_action_impl(
        &app,
        "look around".to_string(),
    );
    let after_first = latest_state(&app);
    let first_narration_count = after_first
        .narrative
        .history()
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert_eq!(first_narration_count, 1);

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let after_retry = latest_state(&app);
    let retry_narration_count = after_retry
        .narrative
        .history()
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        retry_narration_count, 1,
        "Retry should replace old narration, not append another"
    );
}

#[test]
fn test_retry_with_empty_history_is_noop() {
    let app = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let final_state = latest_state(&app);
    assert!(final_state.narrative.history().is_empty());
}

#[test]
fn test_retry_after_llm_failure_succeeds() {
    let msg = Message::new(
        Some("Player".to_string()),
        "look",
        MessageType::Input,
        None,
        None,
    );
    let failing_app = SqliteTestAppBuilder::default_test()
        .message(msg)
        .separate_backends(|| MockBackend::default().with_fail(), MockBackend::default)
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::execute_action_impl(
        &failing_app,
        "look".to_string(),
    );
    let after_fail = latest_state(&failing_app);
    assert!(
        after_fail
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
    );

    let working_app = TestAppBuilder::from_base(
        &failing_app,
        Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            chronicler_engine::application::agents::registry::AgentRegistry::default(),
        )),
    );
    chronicler_engine::application::action_pipeline::retry_last_response_impl(&working_app);

    let after_retry = latest_state(&working_app);
    assert!(
        !after_retry.narrative.input_buffer.status.is_generating(),
        "Retry should complete: {:?}",
        after_retry.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_no_snapshot() {
    let app = make_test_app_without_snapshot(create_test_state()).unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let guard = latest_state(&app);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry without snapshot should not hang in generating state"
    );
}

#[test]
fn test_retry_no_input_text() {
    let msgs = vec![
        Message::new(None, "System boot", MessageType::System, None, None),
        Message::new(None, "You see a room.", MessageType::Narration, None, None),
    ];
    let app = SqliteTestAppBuilder::default_test()
        .messages(msgs)
        .mock_backend(MockBackend::default)
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let guard = latest_state(&app);
    assert_eq!(guard.narrative.history().len(), 2);
}

#[test]
fn test_retry_room_not_found() {
    let data = chronicler_engine::test_support::TestDataBuilder::default_test().build();
    let app = SqliteTestAppBuilder::with_data(data)
        .mock_backend(MockBackend::default)
        .state_mut(|state| {
            state.add_message(
                "look around".to_string(),
                Some("Player".to_string()),
                MessageType::Input,
            );
            state.movement.current_room_id = "non_existent_room".to_string();
        })
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let guard = latest_state(&app);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("Room not found")
        ),
        "Expected room not found error: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_llm_error() {
    let msg = Message::new(
        Some("Player".to_string()),
        "look around",
        MessageType::Input,
        None,
        None,
    );
    let app = SqliteTestAppBuilder::default_test()
        .message(msg)
        .mock_backend(|| MockBackend::default().with_fail())
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let guard = latest_state(&app);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Expected error status after failing LLM: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_empty_narration() {
    let msg = Message::new(
        Some("Player".to_string()),
        "look around",
        MessageType::Input,
        None,
        None,
    );
    let app = SqliteTestAppBuilder::default_test()
        .message(msg)
        .mock_backend(|| MockBackend::default().with_empty_response())
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let guard = latest_state(&app);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")
        ),
        "Expected empty response error: {:?}",
        guard.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_main_narration_uses_pre_main_snapshot() {
    let msg = Message::new(
        Some("Player".to_string()),
        "look around",
        MessageType::Input,
        None,
        None,
    );
    let app = SqliteTestAppBuilder::default_test()
        .message(msg)
        .generation_status(GenerationStatus::Idle, GenerationPhase::default())
        .mock_backend(MockBackend::default)
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let completed = wait_for_generation_complete(&app, 1000);
    assert!(completed, "Retry should complete within timeout");

    let guard = latest_state(&app);
    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(!narrations.is_empty(), "Retry should generate narration");
}

#[test]
fn test_retry_event_continuation_uses_pre_event_snapshot() {
    let data = trigger_data();

    let state_for_closure = {
        let mut s = create_test_state_with_trigger_npc();
        s.narrative.history.clear();
        s.add_message(
            "look around".to_string(),
            Some("Player".to_string()),
            MessageType::Input,
        );
        s.add_message(
            "You look around the shop.".to_string(),
            None,
            MessageType::Narration,
        );
        s.narrative.pending_event = Some("Greeting".to_string());
        s.add_message(
            "The shopkeeper looks up with a smile.".to_string(),
            None,
            MessageType::Narration,
        );
        s.narrative.input_buffer.status = GenerationStatus::Idle;
        s.narrative.last_trigger = Some(StoredTriggerContext {
            npc_id: "shopkeeper".to_string(),
            trigger_idx: 0,
            trigger_name: "Greeting".to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: "The shopkeeper looks up with a smile.".to_string(),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        });
        s
    };

    let app = SqliteTestAppBuilder::with_data(data)
        .game_service_fn(move |storage| {
            let pre_event = GameStateSnapshot::from_game_state(&state_for_closure);
            let pre_event_id = storage.save_snapshot(&pre_event).unwrap();

            let mut cloned = state_for_closure.clone();
            if let Some(last) = cloned.narrative.history.last_mut() {
                last.set_event_header(Some("Greeting".to_string()));
            }

            let final_snap = GameStateSnapshot::from_game_state(&cloned);
            let _ = storage.save_snapshot(&final_snap);

            for mut msg in cloned.narrative.history.iter().cloned().collect::<Vec<_>>() {
                if msg.message_type == MessageType::Narration && msg.event_header().is_none() {
                    msg.set_snapshot_id(Some(pre_event_id));
                }
                let _ = storage.insert_message(&msg);
            }

            Arc::new(GameService::with_mock_quantifier(
                crate::make_test_recorder(Arc::new(MockBackend::default())),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let completed = wait_for_generation_complete(&app, 1000);
    assert!(completed, "Event retry should complete within timeout");

    let guard = latest_state(&app);
    let main_narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        !main_narrations.is_empty(),
        "Should have at least one narration after retry"
    );
}
