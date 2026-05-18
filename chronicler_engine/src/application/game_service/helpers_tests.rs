use crate::application::game_service::context::GameServiceContext;
use crate::application::game_service::helpers::{
    load_state, map_llm_error, save_committed_state, save_state,
};
use crate::error::{EngineError, LlmFailure, NarrativeFailure};
use crate::model::message::Message;
use crate::model::state::GameState;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};
use crate::test_support::in_memory_storage::InMemoryGameStorage;
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
    crate::model::state::GameStateBuilder::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        "start",
    )
    .build()
}

fn minimal_ctx() -> GameServiceContext {
    let state = minimal_state();
    let storage = Arc::new(InMemoryGameStorage::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> =
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>;
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> =
        Arc::clone(&storage) as Arc<dyn crate::storage::message_storage::MessageStorage>;
    let _ = snapshot_storage
        .save(&crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state));
    GameServiceContext {
        snapshot_storage,
        message_storage,
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        action_lock: Arc::new(std::sync::Mutex::new(())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
    }
}

#[test]
fn test_load_state_hydrates_messages() {
    let ctx = minimal_ctx();
    let msg = Message {
        id: 1,
        sender: Some("System".to_string()),
        text: "Hello".to_string(),
        log_type: crate::model::state::LogType::System,
        timestamp: chrono::Utc::now(),
        location_header: None,
        event_header: None,
        snapshot_id: None,
    };
    ctx.message_storage
        .insert_message(&mut msg.clone())
        .unwrap();

    let state = load_state(&ctx);
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text, "Hello");
}

#[test]
fn test_load_state_fallback_when_empty() {
    let mut state = minimal_state();
    state.movement.current_room_id = "other".to_string();
    let storage = Arc::new(InMemoryGameStorage::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> =
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>;
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> =
        Arc::clone(&storage) as Arc<dyn crate::storage::message_storage::MessageStorage>;
    let ctx = GameServiceContext {
        snapshot_storage,
        message_storage,
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        action_lock: Arc::new(std::sync::Mutex::new(())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
    };
    let loaded = load_state(&ctx);
    assert_eq!(loaded.movement.current_room_id, "start");
}

#[test]
fn test_save_and_save_committed_state() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();
    state.movement.current_room_id = "room2".to_string();
    let id = save_state(&ctx, &mut state).unwrap();
    assert!(id > 0);

    let committed_id = save_committed_state(&ctx, &mut state).unwrap();
    assert!(committed_id > id);

    let loaded = ctx
        .snapshot_storage
        .load_by_id(committed_id)
        .unwrap()
        .unwrap();
    assert!(loaded.committed);
    assert!(loaded.committed);
}
