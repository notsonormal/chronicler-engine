//! [DOC: docs/system/game_flow.md]
//! Application context and state management

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::error::{EngineError, LlmFailure};
use crate::domain::model::character::NpcCard;
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;
use crate::narrative::prompt::assembler::assemble_prompt_text;
use crate::adapters::driven::storage::Storage;

#[derive(Clone)]
pub struct GameServiceContext {
    pub storage: Arc<Storage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<crate::domain::model::map::MapDef>,
    pub player: Arc<crate::domain::model::character::PlayerCard>,
    pub npcs: Arc<std::collections::HashMap<String, NpcCard>>,
    pub cancel_token: CancellationToken,
    /// Whether an async generation is in flight.
    pub is_generating: Arc<AtomicBool>,
    /// Runtime settings (shared with [`AppState`]).
    pub settings: Arc<RwLock<AppSettings>>,
    pub preset_storage: Arc<Storage>,
}

impl GameServiceContext {
    pub fn set_game_id(&self, game_id: u64) {
        self.storage.set_game_id(game_id);
    }

    pub fn load_messages(
        &self,
    ) -> Result<Vec<crate::domain::model::message::Message>, EngineError> {
        load_messages_with_swipes(&self.storage)
    }

    pub fn update_message_text(&self, id: u64, text: &str) -> Result<(), EngineError> {
        let index = self.storage.get_active_swipe_index(id)?;
        self.storage.update_swipe_text(id, index, text)
    }

    pub fn active_quantifier_prompt(&self) -> String {
        let preset_id = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            settings.active_quantifier_prompt_preset_id.clone()
        };
        match self.preset_storage.get_preset(&preset_id) {
            Ok(Some(preset)) => assemble_prompt_text(&preset, &[], None),
            Ok(None) => {
                tracing::error!(
                    "active quantifier preset '{preset_id}' not found — defaults not seeded?"
                );
                String::new()
            }
            Err(e) => {
                tracing::error!("preset storage inaccessible: {e}");
                String::new()
            }
        }
    }

    /// Returns (index, anchor message, snapshot id) for retry targeting.
    pub fn find_retry_anchor<'a>(
        &self,
        messages: &'a [Message],
    ) -> Option<(usize, &'a Message, u64)> {
        if messages.is_empty() {
            return None;
        }
        let is_event = messages
            .last()
            .map(|m| m.event_header().is_some())
            .unwrap_or(false);
        let anchor_idx = if is_event {
            messages.iter().rposition(|m| m.event_header().is_none())?
        } else {
            messages
                .iter()
                .rposition(|m| m.message_type == MessageType::Input)?
        };
        let anchor_msg = &messages[anchor_idx];
        let snapshot_id = *anchor_msg.snapshot_id().as_ref()?;
        Some((anchor_idx, anchor_msg, snapshot_id))
    }

    #[cfg(test)]
    pub fn load_state_for_test(&self) -> GameState {
        let snapshot = match self.storage.load_latest_snapshot() {
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

pub fn load_messages_with_swipes(
    storage: &Storage,
) -> Result<Vec<crate::domain::model::message::Message>, EngineError> {
    let mut messages = storage.load_message_rows()?;
    let ids: Vec<u64> = messages.iter().map(|m| m.id).collect();
    let swipes_map = storage.load_swipes_for_messages(&ids)?;
    for msg in &mut messages {
        if let Some(swipes) = swipes_map.get(&msg.id) {
            msg.swipes = swipes.clone();
            let fallback_applied = msg.ensure_valid_swipe_index();
            if fallback_applied {
                tracing::warn!(
                    "active_swipe_index was out of bounds for message {} ({} swipes), fell back to 0",
                    msg.id,
                    msg.swipes.len()
                );
            }
            msg.set_active_swipe(msg.active_swipe_index);
        }
    }
    Ok(messages)
}

pub fn load_expecting_valid_state(ctx: &GameServiceContext) -> Result<GameState, EngineError> {
    let snapshot = ctx.storage.load_latest_snapshot()?;
    let mut state = match snapshot {
        Some(snap) => GameState::from_snapshot(
            &snap,
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).clone(),
        ),
        None => {
            let starting_room_id = ctx.world.starting_room_id();
            GameState::new(
                Arc::clone(&ctx.world),
                Arc::clone(&ctx.map),
                Arc::clone(&ctx.player),
                (*ctx.npcs).values().cloned().collect(),
                starting_room_id,
            )
        }
    };
    load_messages_into_state(ctx, &mut state);
    Ok(state)
}

pub fn load_or_fresh(ctx: &GameServiceContext) -> GameState {
    match load_expecting_valid_state(ctx) {
        Ok(state) => state,
        Err(e) => {
            tracing::error!(
                "Failed to load game state ({e}), falling back to fresh state. This may indicate data corruption."
            );
            let starting_room_id = ctx.world.starting_room_id();
            GameState::new(
                Arc::clone(&ctx.world),
                Arc::clone(&ctx.map),
                Arc::clone(&ctx.player),
                (*ctx.npcs).values().cloned().collect(),
                starting_room_id,
            )
        }
    }
}

pub fn load_messages_into_state(ctx: &GameServiceContext, state: &mut GameState) {
    if let Ok(msgs) = ctx.load_messages() {
        state.narrative.history.replace(msgs);
    }
}

pub fn save_state(ctx: &GameServiceContext, state: &GameState) -> Result<u64, EngineError> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    ctx.storage.save_snapshot(&snapshot)
}

pub fn save_message_and_snapshot(
    ctx: &GameServiceContext,
    state: &mut GameState,
) -> Result<u64, EngineError> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    let snapshot_id = ctx.storage.save_snapshot(&snapshot)?;

    // Persist new swipe on retry target
    if let Some(ref mut target) = state.narrative.retry_target {
        let idx = target.swipes.len().saturating_sub(1);
        if let Some(last_swipe) = target.swipes.last_mut() {
            if last_swipe.snapshot_id.is_none() {
                last_swipe.snapshot_id = Some(snapshot_id);
                ctx.storage.insert_swipe(target.id, last_swipe, idx)?;
                ctx.storage.update_active_swipe(target.id, idx)?;
            }
        }
    }

    if let Some(msg) = state.narrative.history.last_mut() {
        if msg.is_unpersisted() {
            msg.set_snapshot_id(Some(snapshot_id));
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.snapshot_id = Some(snapshot_id);
            }
            let id = ctx.storage.insert_message(&*msg)?;
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                ctx.storage.insert_swipe(id, swipe, idx)?;
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
    ctx.storage.delete_message(id)?;
    state.narrative.history.retain(|m| m.id != id);
    Ok(())
}

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
