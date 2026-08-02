//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! MessageService — owns `Arc<Storage>` and the deep game-state lifecycle seam:
//! message history + snapshot operations (load, save, retry anchor, swipes,
//! edit, delete).

use std::sync::Arc;

use chrono::Utc;

use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::application::errors::ApplicationError;
use crate::domain::model::message::Message;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::template::render_template;
use crate::error::EngineError;

pub struct MessageService {
    storage: Arc<Storage>,
}

impl MessageService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn load_or_fresh(&self) -> GameState {
        match self.load_expecting_valid_state() {
            Ok(state) => state,
            Err(e) => {
                tracing::error!(
                    "Failed to load game state ({e}), building fresh initial state. This may indicate data corruption."
                );
                match self.build_fresh_initial_state() {
                    Ok(state) => state,
                    Err(e2) => {
                        tracing::error!(
                            "build_fresh_initial_state also failed: {e2}; returning empty GameState"
                        );
                        GameState::new("")
                    }
                }
            }
        }
    }

    pub fn load_expecting_valid_state(&self) -> Result<GameState, EngineError> {
        let snapshot = self.storage.load_latest_snapshot()?;
        let snap_data = snapshot.ok_or_else(|| {
            EngineError::Config("no game state snapshot found for current game".to_string())
        })?;
        let mut state = GameState::from_snapshot(&snap_data);
        self.load_messages_into_state(&mut state);
        Ok(state)
    }

    pub fn save_state(&self, state: &GameState) -> Result<u64, EngineError> {
        self.write_snapshot(state)
    }

    pub fn save_message_and_snapshot(&self, state: &mut GameState) -> Result<u64, EngineError> {
        let snapshot_id = self.write_snapshot(state)?;

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

        for msg in state.narrative.history.iter_mut() {
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

    fn write_snapshot(&self, state: &GameState) -> Result<u64, EngineError> {
        let snapshot = GameStateSnapshot::from_game_state(state);
        self.storage.save_snapshot(&snapshot)
    }

    pub fn load_messages(&self) -> Result<Vec<Message>, EngineError> {
        self.storage.load_messages_with_swipes()
    }

    pub fn load_messages_into_state(&self, state: &mut GameState) {
        if let Ok(msgs) = self.load_messages() {
            state.narrative.history.replace(msgs);
        }
    }

    pub fn build_fresh_initial_state(&self) -> Result<GameState, EngineError> {
        let WorldBundle {
            world,
            map,
            persona,
            npcs: npcs_map,
        } = self
            .storage
            .world_bundle_for(self.storage.current_game_id())?;
        let starting_room_id = world.starting_room_id();
        let mut initial_state = GameState::new(starting_room_id.clone());

        if let Some(scenario) = world.default_scenario() {
            let room_name = map
                .get_room_by_id(&starting_room_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| starting_room_id.clone());

            initial_state.narrative.pending_location = Some(room_name);

            let text = render_template(&scenario.text, &TemplateVars::new(&persona.sheet.name));
            if !text.is_empty() {
                initial_state.add_message(text, None, MessageType::Narration);
            }

            initial_state.init_scenario_npcs(scenario, &npcs_map);
        }

        Ok(initial_state)
    }

    fn update_message_text(&self, id: u64, text: &str) -> Result<(), EngineError> {
        let index = self.storage.require_active_swipe_index(id)?;
        self.storage.update_swipe_text(id, index, text)
    }

    pub fn switch_swipe(
        &self,
        is_generating: bool,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError> {
        if is_generating {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let messages = self.load_messages()?;
        let is_last = messages.last().map(|m| m.id == message_id).unwrap_or(false);
        if !is_last {
            return Err(ApplicationError::validation(
                "Only the last message can be swiped",
            ));
        }

        self.storage.update_active_swipe(message_id, swipe_index)?;

        let target_msg = messages
            .iter()
            .find(|m| m.id == message_id)
            .ok_or_else(|| ApplicationError::internal("Message not found"))?;

        let target_swipe = target_msg
            .swipes
            .get(swipe_index)
            .ok_or_else(|| ApplicationError::internal("Swipe index out of bounds"))?;

        let snapshot_id = target_swipe
            .snapshot_id
            .ok_or_else(|| ApplicationError::internal("Swipe has no associated snapshot"))?;

        let mut snapshot = self
            .storage
            .load_snapshot_by_id(snapshot_id)?
            .ok_or_else(|| ApplicationError::internal("Snapshot not found"))?;

        snapshot.created_at = Utc::now();
        self.storage.save_snapshot(&snapshot)?;

        Ok(())
    }

    pub fn edit_history(&self, id: u64, text: String) -> Result<(), ApplicationError> {
        let latest = self.storage.load_latest_snapshot()?;
        let mut guard = self.load_or_fresh();
        guard.narrative.history.edit(id, text.clone())?;

        if latest.is_some() {
            let snapshot = GameStateSnapshot::from_game_state(&guard);
            self.storage.save_snapshot(&snapshot)?;
            self.update_message_text(id, &text)?;
        }

        Ok(())
    }

    pub fn delete_last(&self) -> Result<(), ApplicationError> {
        let mut guard = self.load_or_fresh();
        let last_id = guard
            .narrative
            .history
            .last()
            .map(|m| m.id)
            .ok_or_else(|| ApplicationError::internal("History is empty"))?;

        guard.narrative.history.delete_last()?;
        let snapshot = GameStateSnapshot::from_game_state(&guard);
        self.storage.save_snapshot(&snapshot)?;
        self.storage.delete_message(last_id)?;

        Ok(())
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
}
