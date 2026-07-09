use crate::application::context::{
    OpContext, delete_and_remove_message, load_or_fresh, map_llm_error, save_message_and_snapshot,
    save_state,
};
use crate::error::{EngineError, LlmFailure, NarrativeFailure};
use crate::domain::model::message::Message;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::state::game_state::{GameState, GameStateBuilder};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};
use std::sync::Arc;

#[test]
fn test_map_llm_error_timeout() {
    assert_eq!(
        map_llm_error(&EngineError::Llm(LlmFailure::Timeout)),
        "LLM Error: request timed out"
    );
}

#[test]
fn test_map_llm_error_network() {
    assert_eq!(
        map_llm_error(&EngineError::Llm(LlmFailure::Network {
            url: "http://test".to_string(),
            detail: "connection refused".to_string(),
        })),
        "LLM Error: network error (http://test) — connection refused"
    );
}

#[test]
fn test_map_llm_error_parse() {
    assert_eq!(
        map_llm_error(&EngineError::Llm(LlmFailure::ParseError {
            expected_format: "json",
            raw_response: "bad".to_string(),
        })),
        "LLM Error: unexpected response format (expected json)"
    );
}

#[test]
fn test_map_llm_error_empty() {
    assert_eq!(
        map_llm_error(&EngineError::Llm(LlmFailure::EmptyResponse)),
        "LLM Error: empty response"
    );
}

#[test]
fn test_map_llm_error_http() {
    assert_eq!(
        map_llm_error(&EngineError::Llm(LlmFailure::Http {
            status: 500,
            body: "server error".to_string(),
        })),
        "LLM Error: HTTP 500 — server error"
    );
}

#[test]
fn test_map_llm_error_narrative() {
    assert_eq!(
        map_llm_error(&EngineError::Narrative(NarrativeFailure::Generation {
            stage: "test",
            reason: "fail",
        })),
        "LLM Error: Narration generation failed at stage 'test': fail"
    );
}

#[test]
fn test_map_llm_error_fallback() {
    assert_eq!(
        map_llm_error(&EngineError::Config("bad config".to_string())),
        "LLM Error: Configuration error: bad config"
    );
}

fn minimal_state() -> GameState {
    GameStateBuilder::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        "start",
    )
    .build()
}

fn minimal_ctx() -> OpContext {
    let state = minimal_state();
    let storage = Arc::new(Storage::new_in_memory());
    let _ = storage.save_snapshot(
        &crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        ),
    );
    OpContext {
        storage,
        world_snapshot: crate::application::application_service::WorldSnapshot {
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    }
}

#[test]
fn test_load_or_fresh_hydrates_messages() {
    let ctx = minimal_ctx();
    let msg = Message::new(
        Some("System".to_string()),
        "Hello",
        MessageType::System,
        None,
        None,
    );
    ctx.storage.insert_message(&msg).unwrap();

    let state = load_or_fresh(&ctx);
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text(), "Hello");
}

#[test]
fn test_load_or_fresh_fallback_when_empty() {
    let mut state = minimal_state();
    state.movement.current_room_id = "other".to_string();
    let storage = Arc::new(Storage::new_in_memory());
    let ctx = OpContext {
        storage,
        world_snapshot: crate::application::application_service::WorldSnapshot {
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    };
    let loaded = load_or_fresh(&ctx);
    assert_eq!(loaded.movement.current_room_id, "start");
}

#[test]
fn test_save_and_save_message_and_snapshot() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();
    state.movement.current_room_id = "room2".to_string();
    state.add_message("Test message".to_string(), None, MessageType::Narration);
    let id = save_state(&ctx, &state).unwrap();
    assert!(id > 0);

    let msg_id = save_message_and_snapshot(&ctx, &mut state).unwrap();
    assert!(msg_id > id);

    let loaded = ctx.storage.load_snapshot_by_id(msg_id).unwrap().unwrap();
    assert!(loaded.db_id.is_some());
}

#[test]
fn test_save_message_and_snapshot_persists_retry_swipe() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();

    let mut target = Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    target.id = 42; // Simulate DB-assigned ID
    target.swipes[0].snapshot_id = Some(1);

    target.swipes.push(crate::domain::model::message::Swipe {
        text: "Retried narration".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    });
    target.active_swipe_index = 1;
    target.update_active_swipe_text("Retried narration".to_string());

    state.narrative.retry_target = Some(target);

    let snapshot_id = save_message_and_snapshot(&ctx, &mut state).unwrap();

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes[1].snapshot_id, Some(snapshot_id));
}

#[test]
fn test_save_message_and_snapshot_skips_persisted_retry_swipe() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();

    let mut target = Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    target.id = 42;
    target.swipes[0].snapshot_id = Some(1);
    target.swipes.push(crate::domain::model::message::Swipe {
        text: "Retried narration".to_string(),
        snapshot_id: Some(99), // Already persisted
        location_header: None,
        event_header: None,
    });
    target.active_swipe_index = 1;

    state.narrative.retry_target = Some(target);

    let _snapshot_id = save_message_and_snapshot(&ctx, &mut state).unwrap();

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes[1].snapshot_id, Some(99));
}

#[test]
fn test_active_quantifier_prompt_returns_assembled_text() {
    let ctx = minimal_ctx();
    let preset = PromptPreset {
        id: "quant-test".to_string(),
        name: "Quantifier Test".to_string(),
        role: Some("Tester".to_string()),
        instructions: Some("Be precise".to_string()),
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Quantifier,
    };
    ctx.preset_storage.save_preset(&preset).unwrap();

    {
        let mut settings = ctx.settings.write().unwrap();
        settings.active_quantifier_prompt_preset_id = "quant-test".to_string();
    }

    let result = ctx.active_quantifier_prompt();
    assert!(!result.is_empty(), "Should return assembled prompt text");
    assert!(result.contains("Tester"), "Should contain role: {result}");
}

#[test]
fn test_active_quantifier_prompt_missing_preset_returns_empty() {
    let ctx = minimal_ctx();
    let result = ctx.active_quantifier_prompt();
    assert_eq!(
        result, "",
        "Should return empty string when preset not found"
    );
}

#[test]
fn test_active_quantifier_prompt_storage_error_returns_empty() {
    let mut ctx = minimal_ctx();
    let (failing_preset_storage, handle) = Storage::new_in_memory().with_test_failures();
    ctx.preset_storage = Arc::new(failing_preset_storage);
    handle.set("list_presets", TestOverride::config("fail"));
    let result = ctx.active_quantifier_prompt();
    assert_eq!(result, "", "Should return empty string on storage error");
}

#[test]
fn test_delete_and_remove_message_removes_from_storage_and_state() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();
    let msg = Message::new(
        Some("System".to_string()),
        "To be deleted",
        MessageType::System,
        None,
        None,
    );
    let id = ctx.storage.insert_message(&msg).unwrap();
    let mut msg_with_id = msg;
    msg_with_id.id = id;
    state.narrative.history.append(msg_with_id);

    delete_and_remove_message(&ctx, &mut state, id).unwrap();

    assert_eq!(state.narrative.history.len(), 0);
    assert!(ctx.storage.load_message_rows().unwrap().is_empty());
}

#[test]
fn test_load_or_fresh_fallback_on_snapshot_error() {
    let state = minimal_state();
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let storage = Arc::new(failing_storage);
    handle.set(
        "load_latest_snapshot",
        TestOverride::config("test snap error"),
    );
    let ctx = OpContext {
        storage,
        world_snapshot: crate::application::application_service::WorldSnapshot {
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    };

    let loaded = load_or_fresh(&ctx);
    assert_eq!(loaded.movement.current_room_id, "start");
}

#[test]
fn test_save_message_and_snapshot_propagates_snapshot_error() {
    let state = minimal_state();
    let mut state_copy = state.clone();
    state_copy.add_message("Test".to_string(), None, MessageType::Narration);

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let storage = Arc::new(failing_storage);
    handle.set("save_snapshot", TestOverride::config("test snap error"));
    let ctx = OpContext {
        storage,
        world_snapshot: crate::application::application_service::WorldSnapshot {
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    };

    let result = save_message_and_snapshot(&ctx, &mut state_copy);
    assert!(result.is_err(), "Should propagate snapshot storage error");
}
