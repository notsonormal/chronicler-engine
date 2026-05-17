use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use tokio_util::sync::CancellationToken;

use crate::model::character::NpcCard;
use crate::model::settings::AppSettings;
use crate::model::world::WorldCard;

#[cfg(test)]
use crate::application::game_service::helpers::load_messages_into_state;
#[cfg(test)]
use crate::model::state::GameState;
use crate::storage::llm_message_storage::LlmMessageStorage;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;

/// Context required by [`GameService`] to load and persist game state.
#[derive(Clone)]
pub struct GameServiceContext {
    pub snapshot_storage: Arc<dyn SnapshotStorage>,
    pub message_storage: Arc<dyn MessageStorage>,
    pub llm_message_storage: Arc<dyn LlmMessageStorage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<crate::model::map::MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<std::collections::HashMap<String, NpcCard>>,
    pub cancel_token: CancellationToken,
    /// Serialize async action processing to prevent snapshot race conditions.
    pub action_lock: Arc<Mutex<()>>,
    /// Tracks whether an async generation is currently in flight.
    pub is_generating: Arc<AtomicBool>,
    /// Runtime settings (shared with AppState).
    pub settings: Arc<RwLock<AppSettings>>,
}

impl GameServiceContext {
    /// Read prompt-building parameters from settings.
    /// Returns `(response_length, max_context_tokens, max_tokens)`.
    pub fn prompt_build_params(&self) -> (String, u32, Option<u32>) {
        let guard = self.settings.read().unwrap_or_else(|e| e.into_inner());
        let conn = guard.get_narration_connection();
        let response_length = guard.response_length.clone();
        let max_context_tokens = conn
            .map(|c| c.resolve_max_context_tokens())
            .unwrap_or(crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS);
        let max_tokens = conn.and_then(|c| c.max_tokens);
        (response_length, max_context_tokens, max_tokens)
    }

    /// Load the latest game state from snapshot storage.
    /// Panics if no snapshot exists — use only in tests where a snapshot was pre-seeded.
    #[cfg(test)]
    pub fn load_state(&self) -> GameState {
        let snapshot = match self.snapshot_storage.load_latest() {
            Ok(Some(s)) => s,
            Ok(None) => panic!("no snapshots found"),
            Err(e) => panic!("failed to load snapshot: {e}"),
        };
        let mut state = GameState::from_snapshot(
            &snapshot,
            Arc::clone(&self.world),
            Arc::clone(&self.map),
            Arc::clone(&self.player),
            (*self.npcs).clone(),
        );
        load_messages_into_state(self, &mut state);
        state
    }
}
