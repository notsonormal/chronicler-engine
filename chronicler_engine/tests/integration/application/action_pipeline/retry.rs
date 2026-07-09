//! Integration tests for action retry behaviour: re-running the pipeline against the last user input, no-op on empty history, recovery after a previous LLM failure, and the missing-snapshot error path.

use std::sync::Arc;

use crate::{
    fixtures::{create_test_state, create_test_storage_arc},
    pipeline_helpers::{
        create_test_state_with_trigger_npc, latest_state, wait_for_generation_complete,
    },
};
use chronicler_engine::application::GameService;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;
use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::test_support::{
    make_test_app_with_backends, make_test_app_with_game_service, make_test_app_with_mock_backend,
    make_test_app_with_separate_backends, make_test_app_without_snapshot,
};

#[test]
fn test_retry_finds_last_input_and_runs_pipeline() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

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
    let mut state = create_test_state();
    state.narrative.history.clear();
    let app = make_test_app_with_backends(state, MockBackend::default).unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let final_state = latest_state(&app);
    assert!(final_state.narrative.history().is_empty());
}

#[test]
fn test_retry_after_llm_failure_succeeds() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let failing_app = make_test_app_with_separate_backends(
        state,
        || MockBackend::default().with_fail(),
        MockBackend::default,
    )
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

    let working_app = crate::fixtures::app_with_storage_from(
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
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message("System boot".to_string(), None, MessageType::System);
    state.add_message("You see a room.".to_string(), None, MessageType::Narration);

    let app = make_test_app_with_mock_backend(state, MockBackend::default).unwrap();

    chronicler_engine::application::action_pipeline::retry_last_response_impl(&app);

    let guard = latest_state(&app);
    assert_eq!(guard.narrative.history().len(), 2);
}

#[test]
fn test_retry_room_not_found() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    state.movement.current_room_id = "non_existent_room".to_string();

    let app = chronicler_engine::test_support::make_test_app_with_mock_backend(
        state,
        MockBackend::default,
    )
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
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );

    let app = chronicler_engine::test_support::make_test_app_with_mock_backend(state, || {
        MockBackend::default().with_fail()
    })
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
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );

    let pre_main = GameStateSnapshot::from_game_state(&state);
    let storage = create_test_storage_arc(1);
    chronicler_engine::test_support::seed_test_world_into_storage(&storage, &state);
    let pre_main_id = storage.save_snapshot(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Input {
            msg.set_snapshot_id(Some(pre_main_id));
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.snapshot_id = Some(pre_main_id);
            }
            let msg_id = storage.insert_message(&msg).unwrap();
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = storage.insert_swipe(msg_id, swipe, idx);
            }
        }
    }

    let app = chronicler_engine::test_support::make_test_app_with_mock_backend(state, || {
        MockBackend::default().with_empty_response()
    })
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
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    state.narrative.input_buffer.status = GenerationStatus::Idle;

    let pre_main = GameStateSnapshot::from_game_state(&state);
    let storage = create_test_storage_arc(1);
    chronicler_engine::test_support::seed_test_world_into_storage(&storage, &state);
    let pre_main_id = storage.save_snapshot(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Input {
            msg.set_snapshot_id(Some(pre_main_id));
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.snapshot_id = Some(pre_main_id);
            }
            let msg_id = storage.insert_message(&msg).unwrap();
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = storage.insert_swipe(msg_id, swipe, idx);
            }
        }
    }

    let app = chronicler_engine::test_support::make_test_app_with_mock_backend(
        state,
        MockBackend::default,
    )
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
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    state.add_message(
        "You look around the shop.".to_string(),
        None,
        MessageType::Narration,
    );
    state.narrative.pending_event = Some("Greeting".to_string());
    state.add_message(
        "The shopkeeper looks up with a smile.".to_string(),
        None,
        MessageType::Narration,
    );
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    state.narrative.last_trigger = Some(StoredTriggerContext {
        npc_id: "shopkeeper".to_string(),
        trigger_idx: 0,
        trigger_name: "Greeting".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "The shopkeeper looks up with a smile.".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });

    let app = make_test_app_with_game_service(state.clone(), |storage| {
        let pre_event = GameStateSnapshot::from_game_state(&state);
        let pre_event_id = storage.save_snapshot(&pre_event).unwrap();

        let mut cloned = state.clone();
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
