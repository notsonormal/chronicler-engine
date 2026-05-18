use std::sync::{Arc, RwLock};

use crate::application::game_service::GameServiceContext;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::in_memory_storage::InMemoryGameStorage;

pub fn make_test_context(state: GameState) -> GameServiceContext {
    // [DOC: docs/architecture/system.md]
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(InMemoryGameStorage::new());
    let _ = storage.save(&snapshot);
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = storage.insert_message(&mut msg);
    }

    let snapshot_storage: Arc<dyn SnapshotStorage> = storage.clone();
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = storage.clone();

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
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(RwLock::new(AppSettings::default())),
    }
}

/// [DOC: docs/reference/testing.md]
pub fn make_test_context_with_sqlite(state: GameState) -> crate::error::Result<GameServiceContext> {
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let db_pool = crate::storage::db::DbPool::new(":memory:")?;
    let storage = Arc::new(crate::storage::snapshot_storage::SqliteGameStorage::new(
        db_pool.clone(),
        1,
    ));
    let llm_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> =
        Arc::new(crate::storage::llm_message_storage::SqliteLlmMessageStorage::new(db_pool));
    let _ = storage.save(&snapshot);
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = storage.insert_message(&mut msg);
    }

    let snapshot_storage: Arc<dyn SnapshotStorage> = storage.clone();
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = storage.clone();

    Ok(GameServiceContext {
        snapshot_storage,
        message_storage,
        llm_message_storage: llm_storage,
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(RwLock::new(AppSettings::default())),
    })
}
