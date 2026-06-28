use std::sync::Arc;

use crate::{
    failing_service,
    fixtures::{create_test_state, create_test_storage_arc},
    pipeline_helpers::{
        create_test_state_with_trigger_npc, latest_state, wait_for_generation_complete,
    },
    working_service,
};
use chronicler_engine::application::GameService;
use chronicler_engine::model::state::generation_status::GenerationStatus;
use chronicler_engine::model::state::message_types::MessageType;
use chronicler_engine::model::state::trigger_context::StoredTriggerContext;
use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::{
    make_test_context_with_sqlite, make_test_context_without_snapshot,
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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.execute_action(ctx.clone(), "look around".to_string());
    let after_first = latest_state(&ctx);
    let first_narration_count = after_first
        .narrative
        .history()
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert_eq!(first_narration_count, 1);

    backend.retry_last_response(ctx.clone());

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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = working_service();

    backend.retry_last_response(ctx.clone());

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
    let ctx = make_test_context_with_sqlite(state).unwrap();
    let failing = failing_service();

    failing.execute_action(ctx.clone(), "look".to_string());
    let after_fail = latest_state(&ctx);
    assert!(
        after_fail
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
    );

    let working = working_service();
    working.retry_last_response(ctx.clone());

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

    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

    let guard = latest_state(&ctx);
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

    let ctx = make_test_context_with_sqlite(state).unwrap();
    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

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

    let pre_main = GameStateSnapshot::from_game_state(&state);
    let storage = create_test_storage_arc(1);
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

    let ctx = chronicler_engine::application::GameServiceContext {
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
        preset_storage: {
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
        },
    };

    let main = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&main);

    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

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

    let pre_main = GameStateSnapshot::from_game_state(&state);
    let storage = create_test_storage_arc(1);
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

    let ctx = chronicler_engine::application::GameServiceContext {
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
        preset_storage: {
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
        },
    };

    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default().with_fail()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

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

    let pre_main = GameStateSnapshot::from_game_state(&state);
    let storage = create_test_storage_arc(1);
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

    let ctx = chronicler_engine::application::GameServiceContext {
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
        preset_storage: {
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
        },
    };

    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default().with_empty_response()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

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

    let pre_main = GameStateSnapshot::from_game_state(&state);
    let storage = create_test_storage_arc(1);
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

    let ctx = chronicler_engine::application::GameServiceContext {
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
        preset_storage: {
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
        },
    };

    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

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

    let ctx = make_test_context_with_sqlite(state.clone()).unwrap();
    let pre_event = GameStateSnapshot::from_game_state(&state);
    let pre_event_id = ctx.storage.save_snapshot(&pre_event).unwrap();

    if let Some(last) = state.narrative.history.last_mut() {
        last.set_event_header(Some("Greeting".to_string()));
    }

    let final_snap = GameStateSnapshot::from_game_state(&state);
    let _ = ctx.storage.save_snapshot(&final_snap);

    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.message_type == MessageType::Narration && msg.event_header().is_none() {
            msg.set_snapshot_id(Some(pre_event_id));
        }
        let _ = ctx.storage.insert_message(&msg);
    }

    let backend = GameService::with_mock_quantifier(
        Arc::new(MockBackend::default()),
        Arc::new(MockBackend::default()),
    );

    backend.retry_last_response(ctx.clone());

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
