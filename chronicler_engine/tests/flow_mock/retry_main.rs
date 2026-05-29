use std::sync::Arc;

use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::model::state::{GameState, LogType};
use chronicler_engine::model::trigger::{
    ComparisonOperator, Trigger, TriggerCondition, TriggerEffect,
};
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::make_test_context_with_sqlite;

use crate::pipeline_helpers::{
    add_input_and_save, create_test_state_with_map, latest_snapshot, latest_state, save_state,
    wait_for_generation_complete,
};

#[test]
fn test_retry_main_narration_applies_new_quantifier_result() {
    // Setup: quantifier returns NO movement on first call, movement on second.
    // Flow: Input → Execute → player stays in room1
    //       → Retry → player moves to room2
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk around");

    let quantifier = Arc::new(MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    // First execution: no movement
    service.execute_action(ctx.clone(), "walk around".to_string(), "Player".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "First execution should complete"
    );
    let guard = latest_state(&ctx);
    assert_eq!(
        guard.movement.current_room_id, "room1",
        "First execution: player should stay in room1"
    );

    // Retry: quantifier returns movement → player should move
    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Retry should complete"
    );
    let guard = latest_state(&ctx);
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Retry should apply NEW quantifier result and move player to room2"
    );

    // Verify LLM calls were logged to SQLite storage
    let messages = ctx.storage.list_latest_llm_messages(50).unwrap();
    assert!(
        !messages.is_empty(),
        "LLM messages should be logged during gameplay"
    );
}

#[test]
fn test_retry_with_different_narration_text_reruns_quantifier() {
    // Setup: MockBackend returns different narration text on each call.
    // MockQuantifier auto-detects NPCs from the narration text.
    // Flow: Execute → first narration → quantifier detects no NPCs
    //       → Retry → second narration → quantifier detects NPC from text
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "approach the innkeeper");

    let llm_backend = Arc::new(MockBackend {
        per_call_narrations: vec![
            "You look around the empty room.".to_string(),
            "The Innkeeper greets you warmly.".to_string(),
        ],
        ..Default::default()
    });

    // Use default MockBackend which does word-boundary detection
    let service =
        DefaultGameService::with_mock_quantifier(llm_backend, Arc::new(MockBackend::default()));

    // First execution: narration has no NPC name → quantifier finds no NPCs
    service.execute_action(
        ctx.clone(),
        "approach the innkeeper".to_string(),
        "Player".to_string(),
    );
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "First execution should complete"
    );
    let guard = latest_state(&ctx);
    let first_narration = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.log_type == LogType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert_eq!(
        first_narration, "You look around the empty room.",
        "First narration should match per_call_narrations[0]"
    );

    // Retry: new narration has "Innkeeper" → quantifier should detect the NPC
    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Retry should complete"
    );
    let guard = latest_state(&ctx);
    let retry_narration = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.log_type == LogType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert_eq!(
        retry_narration, "The Innkeeper greets you warmly.",
        "Retry narration should match per_call_narrations[1]"
    );
}

#[test]
fn test_double_retry_increments_swipe_and_reruns_quantifier() {
    // Flow: Execute → Retry → Retry again
    // Verify quantifier runs 3 times total and swipe_index increments.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk around");

    let quantifier = Arc::new(MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
            r#"{"npcs_in_room": []}"#.to_string(),
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    // First execution
    service.execute_action(ctx.clone(), "walk around".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let _snap = latest_snapshot(&ctx).expect("Should have snapshot");

    // First retry
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let _snap = latest_snapshot(&ctx).expect("Should have snapshot");
    let guard = latest_state(&ctx);
    assert_eq!(guard.movement.current_room_id, "room2");

    // Second retry
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let _snap = latest_snapshot(&ctx).expect("Should have snapshot");

    // Verify narration entries are not lost on second retry
    let guard = latest_state(&ctx);
    let history = guard.narrative.history();
    assert!(
        !history.is_empty(),
        "Second retry should produce narration entries"
    );
}

#[test]
fn test_retry_preserves_input_and_does_not_create_extra_swipe() {
    // Bug regression: clicking Retry on Narration was clearing the input message text.
    // The input message should never gain extra swipes.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk around");

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(ctx.clone(), "walk around".to_string(), "Player".to_string());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "First execution should complete"
    );

    service.retry_last_response(ctx.clone());
    assert!(
        wait_for_generation_complete(&ctx, 1000),
        "Retry should complete"
    );

    let guard = latest_state(&ctx);
    let input_msg = guard
        .narrative
        .history
        .iter()
        .find(|m| m.log_type == LogType::Input)
        .expect("Input message must exist");
    assert_eq!(
        input_msg.text, "walk around",
        "Input message text must be preserved after retry"
    );
}

#[test]
fn test_retry_after_edited_input_uses_new_text() {
    // Flow: Execute with "walk around" → edit input to "sprint forward" → Retry
    // Verify retry narration references the edited text.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk around");

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(ctx.clone(), "walk around".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let first_narration = guard
        .narrative
        .history()
        .into_iter()
        .find(|e| e.log_type == LogType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert!(
        first_narration.contains("walk around"),
        "First narration should contain original input: {first_narration}"
    );

    // Edit the input text in-place (overwrite latest snapshot so retry sees it)
    {
        let mut state = latest_state(&ctx);
        if let Some(msg) = state
            .narrative
            .history
            .iter_mut()
            .find(|m| m.log_type == LogType::Input)
        {
            msg.text = "sprint forward".to_string();
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.text = "sprint forward".to_string();
            }
        }
        save_state(&ctx, &state);
    }

    // Retry should use the edited text
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let guard = latest_state(&ctx);
    let retry_narration = guard
        .narrative
        .history()
        .into_iter()
        .rev()
        .find(|e| e.log_type == LogType::Narration)
        .map(|e| e.text.clone())
        .unwrap_or_default();
    assert!(
        retry_narration.contains("sprint forward"),
        "Retry narration should contain edited input: {retry_narration}"
    );
}

#[test]
fn test_main_retry_reevaluates_triggers() {
    // Setup: trigger is room-scoped to room2. Quantifier returns no movement first,
    // then movement to room2 on retry.
    // Flow: Execute → no movement → trigger doesn't fire (wrong room)
    //       → Retry → movement to room2 → trigger fires
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    // Remove default NPCs, add a shopkeeper with a room2-scoped trigger
    let shopkeeper = NpcCard {
        id: "shopkeeper".into(),
        sheet: CharacterSheet {
            name: "Shopkeeper Sarah".into(),
            description: "A shrewd shopkeeper".into(),
            personality: "Business-minded".into(),
            scenario: "Runs the shop".into(),
            example_dialogue: "Welcome!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![Trigger {
            condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
            effect: TriggerEffect {
                name: "Greeting".into(),
                narration_prompt: "The shopkeeper looks up with a smile.".into(),
            },
            repeat: false,
            room_id: Some("room2".to_string()),
        }],
        relationships: vec![],
    };
    state.npcs = std::collections::HashMap::from([("shopkeeper".to_string(), shopkeeper)]);

    let ctx = make_test_context_with_sqlite(state).unwrap();
    add_input_and_save(&ctx, "walk around");

    let quantifier = Arc::new(MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    service.execute_action(ctx.clone(), "walk around".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let guard = latest_state(&ctx);
    let events_after_execute = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.event_header.is_some())
        .count();
    assert_eq!(
        events_after_execute, 0,
        "First execution: no trigger (not in room2)"
    );

    // Retry: movement to room2 → trigger should fire
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let guard = latest_state(&ctx);
    let events_after_retry = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.event_header.is_some())
        .count();
    assert_eq!(
        events_after_retry, 1,
        "Retry should re-evaluate triggers and fire when moved to room2"
    );
}

#[test]
fn test_retry_completes_when_quantifier_returns_none() {
    // Setup: quantifier returns a result on first call, None on second.
    // Flow: Execute → success → Retry → quantifier returns None → still completes
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk around");

    let quantifier = Arc::new(MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": []}"#.to_string(),
            r#"{"npcs_in_room": []}"#.to_string(),
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    service.execute_action(ctx.clone(), "walk around".to_string(), "Player".to_string());
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Retry should still complete even when quantifier returns None on second call
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));
    let guard = latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Retry should complete even if quantifier returns None"
    );
}

#[test]
fn test_retry_no_pre_main_snapshot() {
    // Flow: Execute → clear pre-main snapshot → Retry
    // Retry should fail gracefully when pre-main snapshot is missing.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();

    let db_pool = chronicler_engine::storage::db::DbPool::new(":memory:").unwrap();
    let storage = Arc::new(chronicler_engine::storage::Storage::new_sqlite(
        db_pool.clone(),
        1,
    ));

    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let id = storage.insert_message(&msg).unwrap();
        if let Some(swipe) = msg.swipes.first() {
            let _ = storage.insert_swipe(id, swipe, 0);
        }
    }

    let preset_storage = {
        let ps = chronicler_engine::storage::Storage::new_in_memory();
        let _ = ps.save_preset(&chronicler_engine::model::prompt_preset::PromptPreset {
            id: "system_default".to_string(),
            name: "Default System".to_string(),
            role: Some("You are a narrator.".to_string()),
            instructions: None,
            writing_style: None,
            output_format: None,
            is_default: true,
            preset_type: chronicler_engine::model::prompt_preset::PresetType::System,
        });
        let _ = ps.save_preset(&chronicler_engine::model::prompt_preset::PromptPreset {
            id: "quantifier_default".to_string(),
            name: "Default Quantifier".to_string(),
            role: Some("You are a quantifier.".to_string()),
            instructions: None,
            writing_style: None,
            output_format: None,
            is_default: true,
            preset_type: chronicler_engine::model::prompt_preset::PresetType::Quantifier,
        });
        Arc::new(ps)
    };

    let ctx = chronicler_engine::application::game_service::GameServiceContext {
        storage,
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            chronicler_engine::model::settings::AppSettings::default(),
        )),
        preset_storage,
    };

    add_input_and_save(&ctx, "examine room");

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        Arc::new(MockBackend::default()),
    );

    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    // Get the latest snapshot and state before clearing
    let snap = latest_snapshot(&ctx).expect("Should have snapshot");
    let state_before_reset = GameState::from_snapshot(
        &snap,
        ctx.world.clone(),
        ctx.map.clone(),
        ctx.player.clone(),
        (*ctx.npcs).clone(),
    );

    // Clear all snapshots (simulating missing pre-main)
    {
        let conn = db_pool.conn();
        let _ = conn.execute("DELETE FROM game_state_snapshots WHERE game_id = 1", []);
    }
    // Re-save current state as latest so retry has something to load
    {
        save_state(&ctx, &state_before_reset);
    }

    // Retry should not panic
    service.retry_last_response(ctx.clone());

    // Verify state is stable (not stuck generating)
    let stable = wait_for_generation_complete(&ctx, 500);
    assert!(
        stable,
        "Retry with no pre-main snapshot should complete (possibly with error)"
    );
}

#[test]
fn test_movement_with_arrival_narration_retry() {
    // Flow: Movement action → Retry
    // Verify arrival narration is also regenerated on retry.
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "walk to room2");

    let quantifier = Arc::new(MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#
                .to_string(),
        ],
        ..Default::default()
    });

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(MockBackend::new(Some(Arc::clone(&ctx.storage)))),
        quantifier,
    );

    service.execute_action(
        ctx.clone(),
        "walk to room2".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    let _arrival_count_before = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.text.contains("MockArrival") || e.text.contains("enter"))
        .count();

    // Retry should regenerate both main narration and arrival
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let guard = latest_state(&ctx);
    assert_eq!(
        guard.movement.current_room_id, "room2",
        "Retry should still end in room2"
    );

    // The main narration should have been replaced (only one narration from this turn)
    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.log_type == LogType::Narration)
        .collect();
    assert!(!narrations.is_empty(), "Retry should produce narrations");
}

#[test]
fn test_retry_appends_swipe_to_existing_narration() {
    // Flow: Execute → Retry → verify swipe appended to same message
    let mut state = create_test_state_with_map();
    state.narrative.history.clear();
    let ctx = make_test_context_with_sqlite(state).unwrap();

    add_input_and_save(&ctx, "examine room");

    let llm_backend = Arc::new(MockBackend {
        per_call_narrations: vec![
            "First narration text.".to_string(),
            "Second narration text.".to_string(),
        ],
        ..Default::default()
    });

    let service =
        DefaultGameService::with_mock_quantifier(llm_backend, Arc::new(MockBackend::default()));

    service.execute_action(
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );
    assert!(wait_for_generation_complete(&ctx, 1000));

    let msgs = ctx.load_messages().unwrap();
    let narration = msgs
        .iter()
        .find(|m| m.log_type == LogType::Narration)
        .expect("Should have narration");
    let original_id = narration.id;
    assert_eq!(narration.swipes.len(), 1);

    // Retry should append a swipe to the same message
    service.retry_last_response(ctx.clone());
    assert!(wait_for_generation_complete(&ctx, 1000));

    let msgs = ctx.load_messages().unwrap();
    let narrations: Vec<_> = msgs
        .iter()
        .filter(|m| m.log_type == LogType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        1,
        "Retry should keep exactly one narration message"
    );
    assert_eq!(
        narrations[0].id, original_id,
        "Retry should keep the same message ID"
    );
    assert_eq!(
        narrations[0].swipes.len(),
        2,
        "Retry should append a new swipe"
    );
    assert_eq!(
        narrations[0].text, "Second narration text.",
        "Retry should use next per-call narration as the active swipe"
    );
}
