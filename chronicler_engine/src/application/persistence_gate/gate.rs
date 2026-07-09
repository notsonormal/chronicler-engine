//! [DOC: docs/system/game_flow.md]
//! PersistenceGate — owns game `Arc<Storage>` + `Arc<PresetStore>` + persistence helpers
//! (T2 ticket 02 — façade-first carve-out from DefaultApplicationService).

use std::sync::Arc;

use crate::adapters::driven::storage::PresetStore;
use crate::adapters::driven::storage::Storage;
use crate::domain::model::character::NpcCard;
use crate::domain::model::message::Message;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::template::{render_template, TemplateVars};
use crate::error::EngineError;

use super::dto::WorldSnapshot;

#[derive(Clone)]
pub struct PersistenceGate {
    storage: Arc<Storage>,
    preset_store: Arc<PresetStore>,
}

impl PersistenceGate {
    pub fn new(storage: Arc<Storage>, preset_store: Arc<PresetStore>) -> Self {
        Self {
            storage,
            preset_store,
        }
    }

    pub fn storage(&self) -> &Arc<Storage> {
        &self.storage
    }

    pub fn preset_store(&self) -> &Arc<PresetStore> {
        &self.preset_store
    }

    pub(crate) fn load_world_snapshot(&self) -> Result<WorldSnapshot, EngineError> {
        let game_id = self.storage.current_game_id();
        let game = self
            .storage
            .get_game(game_id)?
            .ok_or_else(|| EngineError::Config(format!("current_game_id {game_id} not found")))?;
        let world_with_map = self
            .storage
            .get_world(&game.world_key)?
            .ok_or_else(|| EngineError::Config(format!("world '{}' not found", game.world_key)))?;
        let player = self
            .storage
            .get_persona(&game.persona_key)?
            .ok_or_else(|| {
                EngineError::Config(format!("persona '{}' not found", game.persona_key))
            })?;
        let npcs_list = self.storage.list_characters(world_with_map.world_id)?;
        let mut npcs: std::collections::HashMap<String, NpcCard> = std::collections::HashMap::new();
        for n in npcs_list {
            npcs.insert(n.id.clone(), n);
        }
        Ok(WorldSnapshot {
            world: Arc::new(world_with_map.world_card),
            map: Arc::new(world_with_map.map),
            player: Arc::new(player),
            npcs: Arc::new(npcs),
        })
    }

    fn world_snapshot_or_empty(&self) -> WorldSnapshot {
        self.load_world_snapshot().unwrap_or_else(|e| {
            tracing::warn!("load_world_snapshot: falling back to empty world snapshot: {e}");
            WorldSnapshot::empty()
        })
    }

    pub fn load_or_fresh(&self) -> Result<GameState, EngineError> {
        match self.load_expecting_valid_state() {
            Ok(state) => Ok(state),
            Err(e) => {
                tracing::error!(
                    "Failed to load game state ({e}), falling back to fresh state. This may indicate data corruption."
                );
                let snap = self.world_snapshot_or_empty();
                let starting_room_id = snap.world.starting_room_id();
                Ok(GameState::new(
                    snap.world,
                    snap.map,
                    snap.player,
                    (*snap.npcs).values().cloned().collect(),
                    starting_room_id,
                ))
            }
        }
    }

    pub fn load_expecting_valid_state(&self) -> Result<GameState, EngineError> {
        let snap = self.world_snapshot_or_empty();
        let snapshot = self.storage.load_latest_snapshot()?;
        let mut state = match snapshot {
            Some(snap_data) => GameState::from_snapshot(
                &snap_data,
                Arc::clone(&snap.world),
                Arc::clone(&snap.map),
                Arc::clone(&snap.player),
                (*snap.npcs).clone(),
            ),
            None => {
                let starting_room_id = snap.world.starting_room_id();
                GameState::new(
                    Arc::clone(&snap.world),
                    Arc::clone(&snap.map),
                    Arc::clone(&snap.player),
                    (*snap.npcs).values().cloned().collect(),
                    starting_room_id,
                )
            }
        };
        self.load_messages_into_state(&mut state);
        Ok(state)
    }

    pub fn save_state(&self, state: &GameState) -> Result<u64, EngineError> {
        let snapshot = GameStateSnapshot::from_game_state(state);
        self.storage.save_snapshot(&snapshot)
    }

    pub fn save_message_and_snapshot(&self, state: &mut GameState) -> Result<u64, EngineError> {
        let snapshot = GameStateSnapshot::from_game_state(state);
        let snapshot_id = self.storage.save_snapshot(&snapshot)?;

        if let Some(ref mut target) = state.narrative.retry_target {
            let idx = target.swipes.len().saturating_sub(1);
            if let Some(last_swipe) = target.swipes.last_mut() {
                if last_swipe.snapshot_id.is_none() {
                    last_swipe.snapshot_id = Some(snapshot_id);
                    self.storage.insert_swipe(target.id, last_swipe, idx)?;
                    self.storage.update_active_swipe(target.id, idx)?;
                }
            }
        }

        if let Some(msg) = state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                if let Some(swipe) = msg.swipes.first_mut() {
                    swipe.snapshot_id = Some(snapshot_id);
                }
                let id = self.storage.insert_message(&*msg)?;
                for (idx, swipe) in msg.swipes.iter().enumerate() {
                    self.storage.insert_swipe(id, swipe, idx)?;
                }
                msg.id = id;
            }
        }
        Ok(snapshot_id)
    }

    pub fn delete_and_remove_message(
        &self,
        state: &mut GameState,
        id: u64,
    ) -> Result<(), EngineError> {
        self.storage.delete_message(id)?;
        state.narrative.history.retain(|m| m.id != id);
        Ok(())
    }

    pub fn load_messages_with_swipes(&self) -> Result<Vec<Message>, EngineError> {
        crate::application::application_service::load_messages_with_swipes(&self.storage)
    }

    pub fn load_messages_into_state(&self, state: &mut GameState) {
        if let Ok(msgs) = self.load_messages() {
            state.narrative.history.replace(msgs);
        }
    }

    pub fn build_fresh_initial_state(&self) -> Result<GameState, EngineError> {
        let snap = self.world_snapshot_or_empty();
        let starting_room_id = snap.world.starting_room_id();
        let mut initial_state = GameState::new(
            Arc::clone(&snap.world),
            Arc::clone(&snap.map),
            Arc::clone(&snap.player),
            (*snap.npcs).values().cloned().collect(),
            starting_room_id.clone(),
        );

        if let Some(scenario) = snap.world.default_scenario() {
            let room_name = snap
                .map
                .get_room_by_id(&starting_room_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| starting_room_id.clone());

            initial_state.narrative.pending_location = Some(room_name);

            let text = render_template(&scenario.text, &TemplateVars::new(&snap.player.sheet.name));
            if !text.is_empty() {
                initial_state.add_message(text, None, MessageType::Narration);
            }

            initial_state.init_scenario_npcs(scenario);
        }

        Ok(initial_state)
    }

    pub fn load_messages(&self) -> Result<Vec<Message>, EngineError> {
        crate::application::application_service::load_messages_with_swipes(&self.storage)
    }

    pub fn update_message_text(&self, id: u64, text: &str) -> Result<(), EngineError> {
        let index = self.storage.get_active_swipe_index(id)?;
        self.storage.update_swipe_text(id, index, text)
    }

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

    pub fn set_game_id(&self, game_id: u64) {
        self.storage.set_game_id(game_id);
    }
}
