use std::sync::Arc;

use super::*;

#[test]
fn test_delayed_llm_completes_without_deadlock() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(200)),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after delayed action completes"
    );
    assert_eq!(
        guard.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should be reset after completion"
    );
}

#[test]
fn test_quantifier_detects_movement() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "village_square"}}"#.to_string()],
            ..Default::default()
        }),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "walk to the village square".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 500);
    assert!(completed, "Movement action should complete within timeout");

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after movement action"
    );
    assert_ne!(
        guard.movement.current_room_id, "room1",
        "Player should have moved from starting room"
    );
}

#[test]
fn test_quantifier_detects_npc_presence_and_fires_trigger() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "enter the shop".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after trigger action"
    );

    let has_event = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.event_header.is_some());
    assert!(has_event, "Trigger should add an event header");

    let narration_count = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 2,
        "Should have main narration + trigger continuation narration"
    );
}

#[test]
fn test_empty_llm_response_handled_gracefully() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        matches!(
            guard.narrative.input_buffer.status,
            GenerationStatus::Error(ref msg) if msg.contains("empty")
        ),
        "Status should be Error after empty LLM response: {:?}",
        guard.narrative.input_buffer.status
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        !has_narration,
        "Empty narration should NOT be added to history"
    );
}

#[test]
fn test_failing_trigger_narration_does_not_crash() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_failing_trigger_narration()),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the shopkeeper".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be reset after trigger narration failure"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "Main narration should exist even when trigger narration failed"
    );

    let has_trigger_error = guard.narrative.history().into_iter().any(|e| {
        e.message_type == MessageType::System && e.text.contains("Trigger narration failed")
    });
    assert!(
        has_trigger_error,
        "Trigger narration failure should be logged"
    );
}

#[test]
fn test_pipeline_cancels_when_token_cancelled() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    let ctx = make_test_context(state);
    ctx.cancel_token.cancel();
    let backend = working_backend();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = latest_state(&ctx);
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Should reset to Idle when cancelled"
    );
}

#[tokio::test]
async fn test_cancellation_resets_state_to_idle() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::with_delay(50)),
        Arc::new(MockBackend::default()),
    );
    let token = ctx.cancel_token.clone();

    // Cancel before execution
    token.cancel();
    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation"
    );
}

#[tokio::test]
async fn test_pipeline_cancels_after_main_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let mock_narrator = Arc::new(MockBackend::with_delay(50));
    let backend = Arc::new(DefaultGameService::with_mock_quantifier(
        mock_narrator.clone(),
        Arc::new(MockBackend::default()),
    ));
    let token = ctx.cancel_token.clone();

    let ctx_clone = ctx.clone();
    let backend_clone = Arc::clone(&backend);
    let handle = tokio::task::spawn_blocking(move || {
        execute_action_impl(
            &*backend_clone,
            ctx_clone.clone(),
            "look around".to_string(),
            "Player".to_string(),
        );
    });

    // Wait for the narration call to start before cancelling.
    while !mock_narrator
        .narration_started
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }
    token.cancel();

    handle.await.unwrap();

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-narration checkpoint"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        !has_narration,
        "Narration should be discarded when cancelled after main LLM call"
    );
}

#[tokio::test]
async fn test_pipeline_cancels_during_trigger_continuation() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);
    let mock_narrator = Arc::new(MockBackend::with_trigger_delay(50));
    let backend = Arc::new(DefaultGameService::with_mock_quantifier(
        mock_narrator.clone(),
        Arc::new(MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
            ..Default::default()
        }),
    ));
    let token = ctx.cancel_token.clone();

    let ctx_clone = ctx.clone();
    let backend_clone = Arc::clone(&backend);
    let handle = tokio::task::spawn_blocking(move || {
        execute_action_impl(
            &*backend_clone,
            ctx_clone.clone(),
            "enter the shop".to_string(),
            "Player".to_string(),
        );
    });

    // Wait for the trigger narration to start before cancelling.
    while !mock_narrator
        .trigger_started
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }
    token.cancel();

    handle.await.unwrap();

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-trigger checkpoint"
    );

    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "Main narration should be preserved");

    let has_event = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.event_header.is_some());
    assert!(
        !has_event,
        "Trigger event should be discarded when cancelled after trigger LLM call"
    );
}

#[test]
fn test_pre_main_snapshot_saved_before_narration() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the room".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(completed, "FreeAction should complete within timeout");

    let latest = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_pre_event_snapshot_saved_before_continuation() {
    let mut state = create_test_state_with_trigger_npc();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Idle;
    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("shopkeeper") {
        encounter.times_met = 0;
    }
    let ctx = make_test_context(state);

    let quantifier = MockBackend {
        per_call_prompt_responses: vec![r#"{"npcs_in_room": ["shopkeeper"]}"#.to_string()],
        ..Default::default()
    };

    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(quantifier),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine the shopkeeper".to_string(),
        "Player".to_string(),
    );

    let completed = wait_for_generation_complete(&ctx, 1000);
    assert!(
        completed,
        "FreeAction with trigger should complete within timeout"
    );

    let latest = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    assert!(latest.db_id.is_some(), "snapshot should exist");
}

#[test]
fn test_pipeline_with_quantifier() {
    let mut state = create_test_state();
    state.narrative.history.clear();
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    let ctx = make_test_context(state);
    let backend = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look around".to_string(),
        "Player".to_string(),
    );

    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Should complete with quantifier backend"
    );
    let has_narration = guard
        .narrative
        .history()
        .into_iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "Should produce narration with quantifier");
}
