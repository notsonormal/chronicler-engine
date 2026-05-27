use std::sync::{Arc, RwLock};

use crate::application::game_service::GameServiceContext;
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::game_storage::GameStorage;
use crate::storage::message_storage::MessageStorage;
use crate::storage::message_swipe_storage::MessageSwipeStorage;
use crate::storage::prompt_preset_storage::{InMemoryPromptPresetStorage, PromptPresetStorage};
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::in_memory_storage::{
    InMemoryGameRepository, InMemoryMessageRepository, InMemoryMessageSwipeStorage,
    InMemorySnapshotRepository,
};

pub fn make_test_context(state: GameState) -> GameServiceContext {
    // [DOC: docs/architecture/system.md]
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let game_repo = Arc::new(InMemoryGameRepository::new());
    let snapshot_repo = Arc::new(InMemorySnapshotRepository::new());
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let _ = snapshot_repo.save(&snapshot);
    let swipe_storage = Arc::new(InMemoryMessageSwipeStorage::new());
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = message_repo.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = swipe_storage.insert_swipe(id, swipe, 0);
            }
        }
    }

    build_test_context(state, game_repo, snapshot_repo, message_repo, swipe_storage)
}

/// [DOC: docs/reference/testing.md]
pub fn make_test_context_without_snapshot(state: GameState) -> GameServiceContext {
    let game_repo = Arc::new(InMemoryGameRepository::new());
    let snapshot_repo = Arc::new(InMemorySnapshotRepository::new());
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let swipe_storage = Arc::new(InMemoryMessageSwipeStorage::new());
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = message_repo.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = swipe_storage.insert_swipe(id, swipe, 0);
            }
        }
    }

    build_test_context(state, game_repo, snapshot_repo, message_repo, swipe_storage)
}

fn default_test_preset_storage() -> Arc<InMemoryPromptPresetStorage> {
    let storage = InMemoryPromptPresetStorage::new();
    let _ = storage.save(&PromptPreset {
        id: "system_default".to_string(),
        name: "Default Test System".to_string(),
        role: Some("You are a test narrator.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::System,
    });
    Arc::new(storage)
}

fn build_test_context(
    state: GameState,
    game_storage: Arc<dyn GameStorage>,
    snapshot_storage: Arc<dyn SnapshotStorage>,
    message_storage: Arc<dyn crate::storage::message_storage::MessageStorage>,
    message_swipe_storage: Arc<dyn crate::storage::message_swipe_storage::MessageSwipeStorage>,
) -> GameServiceContext {
    GameServiceContext {
        game_storage,
        snapshot_storage,
        message_storage,
        message_swipe_storage,
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(RwLock::new(AppSettings::default())),
        preset_storage: default_test_preset_storage(),
    }
}

/// [DOC: docs/reference/testing.md]
pub fn make_test_context_with_sqlite(state: GameState) -> crate::error::Result<GameServiceContext> {
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let db_pool = crate::storage::db::DbPool::new(":memory:")?;
    let game_repo = Arc::new(crate::storage::game_storage::SqliteGameRepository::new(
        db_pool.clone(),
        1,
    ));
    let snapshot_repo = Arc::new(
        crate::storage::snapshot_storage::SqliteSnapshotRepository::new(db_pool.clone(), 1),
    );
    let message_repo =
        Arc::new(crate::storage::message_storage::SqliteMessageRepository::new(db_pool.clone(), 1));
    let llm_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> = Arc::new(
        crate::storage::llm_message_storage::SqliteLlmMessageStorage::new(db_pool.clone()),
    );
    let _ = snapshot_repo.save(&snapshot);
    let swipe_repo =
        Arc::new(crate::storage::message_swipe_storage::SqliteMessageSwipeRepository::new(db_pool));
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = message_repo.insert_message(&msg) {
            if let Some(swipe) = msg.swipes.first() {
                let _ = swipe_repo.insert_swipe(id, swipe, 0);
            }
        }
    }

    Ok(GameServiceContext {
        game_storage: game_repo,
        snapshot_storage: snapshot_repo,
        message_storage: message_repo,
        message_swipe_storage: swipe_repo,
        llm_message_storage: llm_storage,
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(RwLock::new(AppSettings::default())),
        preset_storage: default_test_preset_storage(),
    })
}
