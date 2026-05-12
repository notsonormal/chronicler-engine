use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::model::character::NpcCard;
use crate::model::world::WorldCard;

#[cfg(test)]
use crate::model::state::GameState;
use crate::storage::snapshot_storage::SnapshotStorage;

/// Context required by [`GameService`] to load and persist game state.
#[derive(Clone)]
pub struct GameServiceContext {
    pub snapshot_storage: Arc<dyn SnapshotStorage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<crate::model::map::MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<std::collections::HashMap<String, NpcCard>>,
    pub cancel_token: CancellationToken,
    /// Serialize async action processing to prevent snapshot race conditions.
    pub action_lock: Arc<Mutex<()>>,
    /// Tracks whether an async generation is currently in flight.
    pub is_generating: Arc<AtomicBool>,
}

impl GameServiceContext {
    /// Load the latest game state from snapshot storage.
    /// Panics if no snapshot exists — use only in tests where a snapshot was pre-seeded.
    #[cfg(test)]
    pub fn load_state(&self) -> GameState {
        let snapshot = match self.snapshot_storage.load_latest(None) {
            Ok(Some(s)) => s,
            Ok(None) => panic!("no snapshots found"),
            Err(e) => panic!("failed to load snapshot: {e}"),
        };
        GameState::from_snapshot(
            &snapshot,
            Arc::clone(&self.world),
            Arc::clone(&self.map),
            Arc::clone(&self.player),
            (*self.npcs).clone(),
        )
    }
}
