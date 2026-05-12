use std::sync::{Arc, Mutex};

use crate::engine::game_service::GameServiceContext;
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::snapshot_storage::SnapshotStorage;
use crate::test_support::in_memory_storage::InMemorySnapshotStorage;

/// Build a [`GameServiceContext`] from a [`GameState`] for tests.
pub fn make_test_context(state: GameState) -> GameServiceContext {
    let snapshot = GameStateSnapshot::from_game_state(&state, "test".to_string(), 0);
    let storage = Arc::new(InMemorySnapshotStorage::new());
    let _ = storage.save(&snapshot);

    GameServiceContext {
        snapshot_storage: storage,
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        action_lock: Arc::new(Mutex::new(())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}
