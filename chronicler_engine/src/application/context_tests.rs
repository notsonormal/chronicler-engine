use crate::application::context::{
    GameServiceContext, delete_and_remove_message, load_state, map_llm_error,
    save_message_and_snapshot, save_state,
};
use crate::error::{EngineError, LlmFailure, NarrativeFailure};
use crate::model::message::Message;
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::model::state::GameState;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};
use crate::test_support::in_memory_storage::{
    InMemoryMessageRepository, InMemorySnapshotRepository,
};
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
    let snapshot_repo = Arc::new(InMemorySnapshotRepository::new());
    let game_repo = Arc::new(crate::test_support::in_memory_storage::InMemoryGameRepository::new());
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> = snapshot_repo;
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = message_repo;
    let _ = snapshot_storage
        .save(&crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state));
    GameServiceContext {
        game_storage: game_repo,
        snapshot_storage,
        message_storage,
        message_swipe_storage: Arc::new(
            crate::test_support::in_memory_storage::InMemoryMessageSwipeStorage::new(),
        ),
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(
            crate::storage::prompt_preset_storage::InMemoryPromptPresetStorage::new(),
        ),
    }
}

#[test]
fn test_load_state_hydrates_messages() {
    let ctx = minimal_ctx();
    let msg = Message::new(
        Some("System".to_string()),
        "Hello",
        crate::model::state::LogType::System,
        None,
        None,
    );
    ctx.message_storage.insert_message(&msg).unwrap();

    let state = load_state(&ctx);
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text, "Hello");
}

#[test]
fn test_load_state_fallback_when_empty() {
    let mut state = minimal_state();
    state.movement.current_room_id = "other".to_string();
    let game_repo = Arc::new(crate::test_support::in_memory_storage::InMemoryGameRepository::new());
    let snapshot_repo = Arc::new(InMemorySnapshotRepository::new());
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> = snapshot_repo;
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = message_repo;
    let ctx = GameServiceContext {
        game_storage: game_repo,
        snapshot_storage,
        message_storage,
        message_swipe_storage: Arc::new(
            crate::test_support::in_memory_storage::InMemoryMessageSwipeStorage::new(),
        ),
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(
            crate::storage::prompt_preset_storage::InMemoryPromptPresetStorage::new(),
        ),
    };
    let loaded = load_state(&ctx);
    assert_eq!(loaded.movement.current_room_id, "start");
}

#[test]
fn test_save_and_save_message_and_snapshot() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();
    state.movement.current_room_id = "room2".to_string();
    state.add_log(
        "Test message".to_string(),
        None,
        crate::model::state::LogType::Narration,
    );
    let id = save_state(&ctx, &state).unwrap();
    assert!(id > 0);

    let msg_id = save_message_and_snapshot(&ctx, &mut state).unwrap();
    assert!(msg_id > id);

    let loaded = ctx.snapshot_storage.load_by_id(msg_id).unwrap().unwrap();
    assert!(loaded.db_id.is_some());
}


// ── Mock storage that always fails ──────────────────────────────────────────

struct FailingSnapshotStorage;

impl SnapshotStorage for FailingSnapshotStorage {
    fn set_game_id(&self, _game_id: u64) {}
    fn current_game_id(&self) -> u64 { 1 }
    fn save(&self, _snapshot: &crate::model::state_snapshot::GameStateSnapshot) -> Result<u64, crate::error::EngineError> {
        Err(crate::error::EngineError::Config("test snap error".to_string()))
    }
    fn load_latest(&self) -> Result<Option<crate::model::state_snapshot::GameStateSnapshot>, crate::error::EngineError> {
        Err(crate::error::EngineError::Config("test snap error".to_string()))
    }
    fn load_by_id(&self, _id: u64) -> Result<Option<crate::model::state_snapshot::GameStateSnapshot>, crate::error::EngineError> {
        Ok(None)
    }
}

// ── active_quantifier_prompt tests ──────────────────────────────────────────

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
    ctx.preset_storage.save(&preset).unwrap();

    // Point settings at our test preset
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
    // Ensure no quantifier preset is seeded
    let result = ctx.active_quantifier_prompt();
    assert_eq!(result, "", "Should return empty string when preset not found");
}

#[test]
fn test_active_quantifier_prompt_storage_error_returns_empty() {
    use crate::storage::prompt_preset_storage::PromptPresetStorage;

    struct FailingPresetStorage;
    impl PromptPresetStorage for FailingPresetStorage {
        fn list(&self, _preset_type: PresetType) -> Result<Vec<PromptPreset>, crate::error::EngineError> {
            Err(crate::error::EngineError::Config("fail".to_string()))
        }
        fn get(&self, _id: &str) -> Result<Option<PromptPreset>, crate::error::EngineError> {
            Err(crate::error::EngineError::Config("fail".to_string()))
        }
        fn save(&self, _preset: &PromptPreset) -> Result<(), crate::error::EngineError> {
            Ok(())
        }
        fn delete(&self, _id: &str) -> Result<(), crate::error::EngineError> {
            Ok(())
        }
    }

    let mut ctx = minimal_ctx();
    ctx.preset_storage = Arc::new(FailingPresetStorage);
    let result = ctx.active_quantifier_prompt();
    assert_eq!(result, "", "Should return empty string on storage error");
}

// ── delete_and_remove_message test ──────────────────────────────────────────

#[test]
fn test_delete_and_remove_message_removes_from_storage_and_state() {
    let ctx = minimal_ctx();
    let mut state = minimal_state();
    let msg = Message::new(
        Some("System".to_string()),
        "To be deleted",
        crate::model::state::LogType::System,
        None,
        None,
    );
    let id = ctx.message_storage.insert_message(&msg).unwrap();
    let mut msg_with_id = msg;
    msg_with_id.id = id;
    state.narrative.history.append(msg_with_id);

    delete_and_remove_message(&ctx, &mut state, id).unwrap();

    assert_eq!(state.narrative.history.len(), 0);
    assert!(ctx.message_storage.load_message_rows().unwrap().is_empty());
}

// ── load_state error fallback test ──────────────────────────────────────────

#[test]
fn test_load_state_fallback_on_snapshot_error() {
    let state = minimal_state();
    let game_repo = Arc::new(crate::test_support::in_memory_storage::InMemoryGameRepository::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> = Arc::new(FailingSnapshotStorage);
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let ctx = GameServiceContext {
        game_storage: game_repo,
        snapshot_storage,
        message_storage: message_repo,
        message_swipe_storage: Arc::new(
            crate::test_support::in_memory_storage::InMemoryMessageSwipeStorage::new(),
        ),
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(
            crate::storage::prompt_preset_storage::InMemoryPromptPresetStorage::new(),
        ),
    };

    let loaded = load_state(&ctx);
    assert_eq!(loaded.movement.current_room_id, "start");
}

// ── save_message_and_snapshot error path ────────────────────────────────────

#[test]
fn test_save_message_and_snapshot_propagates_snapshot_error() {
    let state = minimal_state();
    let mut state_copy = state.clone();
    state_copy.add_log("Test".to_string(), None, crate::model::state::LogType::Narration);

    let game_repo = Arc::new(crate::test_support::in_memory_storage::InMemoryGameRepository::new());
    let snapshot_storage: Arc<dyn SnapshotStorage> = Arc::new(FailingSnapshotStorage);
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let ctx = GameServiceContext {
        game_storage: game_repo,
        snapshot_storage,
        message_storage: message_repo,
        message_swipe_storage: Arc::new(
            crate::test_support::in_memory_storage::InMemoryMessageSwipeStorage::new(),
        ),
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            crate::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(
            crate::storage::prompt_preset_storage::InMemoryPromptPresetStorage::new(),
        ),
    };

    let result = save_message_and_snapshot(&ctx, &mut state_copy);
    assert!(result.is_err(), "Should propagate snapshot storage error");
}
