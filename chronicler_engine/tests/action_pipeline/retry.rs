use std::sync::Arc;

use super::*;

#[test]
fn test_retry_finds_last_input_and_runs_pipeline() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message(
        "look around".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let ctx = make_test_context(state);
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );
    let after_first = latest_state(&ctx);
    let first_narration_count = after_first
        .narrative
        .history()
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert_eq!(first_narration_count, 1);

    retry_last_response_impl(&backend, ctx.clone());

    let after_retry = latest_state(&ctx);
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
    let ctx = make_test_context(state);
    let backend = working_backend();

    retry_last_response_impl(&backend, ctx.clone());

    let final_state = latest_state(&ctx);
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
    let ctx = make_test_context(state);
    let failing = failing_backend();

    execute_action_impl(
        &failing,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );
    let after_fail = latest_state(&ctx);
    assert!(
        after_fail
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
    );

    let working = working_backend();
    retry_last_response_impl(&working, ctx.clone());

    let after_retry = latest_state(&ctx);
    assert!(
        !after_retry.narrative.input_buffer.status.is_generating(),
        "Retry should complete: {:?}",
        after_retry.narrative.input_buffer.status
    );
}

#[test]
fn test_retry_no_snapshot() {
    let ctx = make_test_context_without_snapshot(create_test_state());

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());
}

#[test]
fn test_retry_no_input_text() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.add_message("System boot".to_string(), None, MessageType::System);
    state.add_message("You see a room.".to_string(), None, MessageType::Narration);

    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
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

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.storage.save_snapshot(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.storage.insert_message(&msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
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

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.storage.save_snapshot(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.storage.insert_message(&msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
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

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.storage.save_snapshot(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.storage.insert_message(&msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let guard = latest_state(&ctx);
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

    let ctx = make_test_context(state.clone());
    let pre_main = GameStateSnapshot::from_game_state(&state);
    let pre_main_id = ctx.storage.save_snapshot(&pre_main).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Input {
            msg.snapshot_id = Some(pre_main_id);
        }
        let _ = ctx.storage.insert_message(&msg);
    }

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&main);

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Retry should complete within timeout");

    let guard = latest_state(&ctx);
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

    let ctx = make_test_context(state.clone());
    let pre_event = GameStateSnapshot::from_game_state(&state);
    let pre_event_id = ctx.storage.save_snapshot(&pre_event).unwrap();

    if let Some(last) = state.narrative.history.last_mut() {
        last.event_header = Some("Greeting".to_string());
    }

    let final_snap = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&final_snap);

    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Narration && msg.event_header.is_none() {
            msg.snapshot_id = Some(pre_event_id);
        }
        let _ = ctx.storage.insert_message(&msg);
    }

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    retry_last_response_impl(&backend, ctx.clone());

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "Event retry should complete within timeout");

    let guard = latest_state(&ctx);
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
