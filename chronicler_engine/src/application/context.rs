use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::error::{EngineError, LlmFailure};
use crate::model::character::NpcCard;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::world::WorldCard;
use crate::storage::game_storage::GameStorage;
use crate::storage::llm_message_storage::LlmMessageStorage;
use crate::storage::message_storage::MessageStorage;
use crate::storage::message_swipe_storage::MessageSwipeStorage;
use crate::storage::prompt_preset_storage::PromptPresetStorage;
use crate::storage::snapshot_storage::SnapshotStorage;

#[derive(Clone)]
pub struct GameServiceContext {
    pub game_storage: Arc<dyn GameStorage>,
    pub snapshot_storage: Arc<dyn SnapshotStorage>,
    pub message_storage: Arc<dyn MessageStorage>,
    pub message_swipe_storage: Arc<dyn MessageSwipeStorage>,
    pub llm_message_storage: Arc<dyn LlmMessageStorage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<crate::model::map::MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<std::collections::HashMap<String, NpcCard>>,
    pub cancel_token: CancellationToken,
    /// Tracks whether an async generation is currently in flight.
    pub is_generating: Arc<AtomicBool>,
    /// Runtime settings (shared with AppState).
    pub settings: Arc<RwLock<AppSettings>>,
    pub preset_storage: Arc<dyn PromptPresetStorage>,
}

impl GameServiceContext {
    /// Set the active game id on all storage modules.
    pub fn set_game_id(&self, game_id: u64) {
        self.game_storage.set_game_id(game_id);
        self.snapshot_storage.set_game_id(game_id);
        self.message_storage.set_game_id(game_id);
    }

    /// Load all messages (with swipes) for the current game.
    pub fn load_messages(&self) -> Result<Vec<crate::model::message::Message>, EngineError> {
        load_messages_with_swipes(&*self.message_storage, &*self.message_swipe_storage)
    }

    /// Update the active swipe's text for the given message.
    pub fn update_message_text(&self, id: u64, text: &str) -> Result<(), EngineError> {
        let index = self.message_storage.get_active_swipe_index(id)?;
        self.message_swipe_storage
            .update_swipe_text(id, index, text)
    }

    /// Migrate pending swipes to a new message after retry.
    pub fn migrate_swipes(
        &self,
        message_id: u64,
        pending_swipes: &[crate::model::message::Swipe],
        new_active_index: usize,
        to_delete: &[u64],
    ) -> Result<(), EngineError> {
        let offset = pending_swipes.len();
        self.message_swipe_storage
            .shift_swipe_indices(message_id, offset)?;
        for (idx, swipe) in pending_swipes.iter().enumerate() {
            self.message_swipe_storage
                .insert_swipe(message_id, swipe, idx)?;
        }
        self.message_storage
            .update_active_swipe(message_id, new_active_index)?;
        for id in to_delete {
            self.message_storage.purge_soft_deleted(&[*id])?;
        }
        Ok(())
    }

    /// Quantifier presets do not include global rules or response length.
    pub fn active_quantifier_prompt(&self) -> String {
        let preset_id = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            settings.active_quantifier_prompt_preset_id.clone()
        };
        match self.preset_storage.get(&preset_id) {
            Ok(Some(preset)) => preset.assemble_prompt_text(&[], None),
            Ok(None) => {
                log::error!(
                    "active quantifier preset '{preset_id}' not found — defaults not seeded?"
                );
                String::new()
            }
            Err(e) => {
                log::error!("preset storage inaccessible: {e}");
                String::new()
            }
        }
    }

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

/// Load message rows and attach swipe data.
/// [DOC: docs/architecture/system.md]
pub fn load_messages_with_swipes(
    message_storage: &dyn MessageStorage,
    message_swipe_storage: &dyn MessageSwipeStorage,
) -> Result<Vec<crate::model::message::Message>, EngineError> {
    let mut messages = message_storage.load_message_rows()?;
    let ids: Vec<u64> = messages.iter().map(|m| m.id).collect();
    let swipes_map = message_swipe_storage.load_swipes_for_messages(&ids)?;
    for msg in &mut messages {
        if let Some(swipes) = swipes_map.get(&msg.id) {
            msg.swipes = swipes.clone();
            if let Some(swipe) = msg
                .swipes
                .get(msg.active_swipe_index)
                .or(msg.swipes.first())
            {
                msg.text = swipe.text.clone();
                msg.location_header = swipe.location_header.clone();
                msg.event_header = swipe.event_header.clone();
                msg.snapshot_id = swipe.snapshot_id;
            }
        }
    }
    Ok(messages)
}

/// [DOC: docs/architecture/system.md]
pub fn try_load_state(ctx: &GameServiceContext) -> Result<GameState, EngineError> {
    let snapshot = ctx.snapshot_storage.load_latest()?;
    let mut state = match snapshot {
        Some(snap) => GameState::from_snapshot(
            &snap,
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).clone(),
        ),
        None => GameState::new(
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).values().cloned().collect(),
            ctx.world.starting_room_id.clone(),
        ),
    };
    load_messages_into_state(ctx, &mut state);
    Ok(state)
}

/// [DOC: docs/architecture/system.md]
pub fn load_state(ctx: &GameServiceContext) -> GameState {
    match try_load_state(ctx) {
        Ok(state) => state,
        Err(_) => GameState::new(
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).values().cloned().collect(),
            ctx.world.starting_room_id.clone(),
        ),
    }
}

pub fn load_messages_into_state(ctx: &GameServiceContext, state: &mut GameState) {
    // [DOC: docs/architecture/system.md]
    if let Ok(msgs) = ctx.load_messages() {
        state.narrative.history.replace(msgs);
    }
}

/// [DOC: docs/architecture/system.md]
pub fn save_state(ctx: &GameServiceContext, state: &GameState) -> Result<u64, EngineError> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    ctx.snapshot_storage.save(&snapshot)
}

/// Save a snapshot and persist the most recent unpersisted message.
/// Because messages are persisted immediately, there is typically
/// only one unpersisted message at a time.
/// [DOC: docs/architecture/system.md]
pub fn save_message_and_snapshot(
    ctx: &GameServiceContext,
    state: &mut GameState,
) -> Result<u64, EngineError> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    let snapshot_id = ctx.snapshot_storage.save(&snapshot)?;
    if let Some(msg) = state.narrative.history.last_mut() {
        if msg.id == 0 {
            msg.snapshot_id = Some(snapshot_id);
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.snapshot_id = Some(snapshot_id);
            }
            let id = ctx.message_storage.insert_message(&*msg)?;
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                ctx.message_swipe_storage.insert_swipe(id, swipe, idx)?;
            }
            msg.id = id;
        }
    }
    Ok(snapshot_id)
}

pub fn delete_and_remove_message(
    ctx: &GameServiceContext,
    state: &mut GameState,
    id: u64,
) -> Result<(), EngineError> {
    // [DOC: docs/architecture/system.md]
    ctx.message_storage.delete_message(id)?;
    state.narrative.history.retain(|m| m.id != id);
    Ok(())
}

/// [DOC: docs/architecture/system.md]
pub fn map_llm_error(e: &EngineError) -> String {
    match e {
        EngineError::Llm(LlmFailure::Timeout) => "LLM Error: request timed out".to_string(),
        EngineError::Llm(LlmFailure::Network { url, detail }) => {
            format!("LLM Error: network error ({url}) \u{2014} {detail}")
        }
        EngineError::Llm(LlmFailure::ParseError {
            expected_format, ..
        }) => {
            format!("LLM Error: unexpected response format (expected {expected_format})")
        }
        EngineError::Llm(LlmFailure::EmptyResponse) => "LLM Error: empty response".to_string(),
        EngineError::Llm(LlmFailure::Http { status, body }) => {
            format!("LLM Error: HTTP {status} \u{2014} {body}")
        }
        EngineError::Narrative(nf) => format!("LLM Error: {nf}"),
        _ => format!("LLM Error: {e}"),
    }
}
