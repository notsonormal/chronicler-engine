use std::sync::{Arc, RwLock};

use crate::application::game_service::GameServiceContext;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::in_memory_storage::{
    InMemoryMessageRepository, InMemorySnapshotRepository,
};

pub fn make_test_context(state: GameState) -> GameServiceContext {
    // [DOC: docs/architecture/system.md]
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let snapshot_repo = Arc::new(InMemorySnapshotRepository::new());
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    let _ = snapshot_repo.save(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = message_repo.insert_message(&msg);
    }

    build_test_context(state, snapshot_repo, message_repo)
}

/// [DOC: docs/reference/testing.md]
pub fn make_test_context_without_snapshot(state: GameState) -> GameServiceContext {
    let snapshot_repo = Arc::new(InMemorySnapshotRepository::new());
    let message_repo = Arc::new(InMemoryMessageRepository::new());
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = message_repo.insert_message(&msg);
    }

    build_test_context(state, snapshot_repo, message_repo)
}

fn build_test_context(
    state: GameState,
    snapshot_storage: Arc<dyn SnapshotStorage>,
    message_storage: Arc<dyn crate::storage::message_storage::MessageStorage>,
) -> GameServiceContext {
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
    let snapshot_repo = Arc::new(
        crate::storage::snapshot_storage::SqliteSnapshotRepository::new(db_pool.clone(), 1),
    );
    let message_repo =
        Arc::new(crate::storage::message_storage::SqliteMessageRepository::new(db_pool.clone(), 1));
    let llm_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> =
        Arc::new(crate::storage::llm_message_storage::SqliteLlmMessageStorage::new(db_pool));
    let _ = snapshot_repo.save(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = message_repo.insert_message(&msg);
    }

    Ok(GameServiceContext {
        snapshot_storage: snapshot_repo,
        message_storage: message_repo,
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
