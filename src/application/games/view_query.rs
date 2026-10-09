//! [DOC: docs/diataxis/reference/game_flow.md]
//! GameViewQuery — read-side queries that don't mutate game state.

use std::sync::Arc;

use crate::adapters::driven::storage::llm_messages::LLM_MESSAGE_RETENTION_LIMIT;
use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::domain::model::agent::Role;
use crate::domain::model::llm_message::LlmMessage;
use crate::application::message_service::MessageService;
use crate::domain::model::state::message_types::MessageEntry;
use crate::error::EngineError;

use crate::application::debug::DebugStateView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleHealth {
    pub role: Role,
    pub backend_model: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct GameViewQuery {
    storage: Arc<Storage>,
    message_service: Arc<MessageService>,
}

impl GameViewQuery {
    pub fn new(storage: Arc<Storage>, message_service: Arc<MessageService>) -> Self {
        Self {
            storage,
            message_service,
        }
    }

    pub fn get_current_room_view(&self) -> Result<(String, Option<String>), ApplicationError> {
        let game_state = self.message_service.load_or_fresh();
        let storage = &self.storage;
        let game_id = storage.current_game_id();
        let game = storage.require_game(game_id)?;
        let world_with_map = storage.require_world(&game.world_key)?;
        let world = world_with_map.world_card;
        let map = world_with_map.map;
        let room = map
            .get_room_by_id(&game_state.movement.current_room_id)
            .or_else(|| {
                game_state
                    .movement
                    .dynamic_rooms
                    .get(&game_state.movement.current_room_id)
            })
            .ok_or_else(|| {
                ApplicationError::from(EngineError::RoomNotFound(
                    "current room not found".to_string(),
                ))
            })?;

        let image_path = room
            .image_path
            .clone()
            .or_else(|| world.default_room_image.clone());

        Ok((room.name.clone(), image_path))
    }

    pub fn get_npc_headshots(
        &self,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        let game_state = self.message_service.load_or_fresh();
        let storage = &self.storage;
        let game_id = storage.current_game_id();
        let game = storage.require_game(game_id)?;
        let world_with_map = storage.require_world(&game.world_key)?;
        let npcs_list = storage.list_characters(world_with_map.world_id)?;
        let npcs: std::collections::HashMap<String, _> = {
            let mut m = std::collections::HashMap::new();
            for n in npcs_list {
                m.insert(n.id.clone(), n);
            }
            m
        };

        let npc_ids: Vec<String> = if scene_only {
            game_state
                .scene
                .npcs_in_area
                .iter()
                .map(|npc| npc.id.clone())
                .collect()
        } else {
            npcs.keys().cloned().collect()
        };

        let npc_data: Vec<(String, String)> = npc_ids
            .iter()
            .filter_map(|id| {
                let npc = npcs.get(id)?;
                let image_path = npc.sheet.preferred_image()?.to_string();
                let name = npc.sheet.name.clone();
                Some((image_path, name))
            })
            .collect();

        Ok(npc_data)
    }

    pub fn get_debug_state(&self) -> Result<DebugStateView, ApplicationError> {
        let game_state = self.message_service.load_or_fresh();

        let history_tail: Vec<MessageEntry> = game_state
            .narrative
            .history()
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect();

        let npc_ids: Vec<String> = game_state
            .scene
            .npcs_in_area
            .iter()
            .map(|npc| npc.id.clone())
            .collect();

        let dynamic_rooms: Vec<String> =
            game_state.movement.dynamic_rooms.keys().cloned().collect();

        let last_error = game_state
            .narrative
            .input_buffer
            .status
            .failure()
            .map(|failure| failure.raw.clone());

        Ok(DebugStateView {
            current_room_id: game_state.movement.current_room_id.clone(),
            npcs_in_area: npc_ids,
            generation_status: game_state.narrative.input_buffer.status.clone(),
            generation_phase: game_state.narrative.input_buffer.phase.clone(),
            npc_encounter_log: game_state.npc_encounter_log.npcs.clone(),
            narration_history_tail: history_tail,
            narration_history_length: game_state.narrative.history().len(),
            dynamic_rooms,
            dynamic_room_count: game_state.movement.dynamic_rooms.len(),
            last_error,
            quantifier_confidence: game_state.scene.quantifier_confidence.clone(),
            backend_name: game_state.narrative.last_backend_name.clone(),
            model_name: game_state.narrative.last_model_name.clone(),
            current_options: game_state.narrative.current_options.clone(),
        })
    }

    pub fn get_story_log_entries(&self) -> Result<(Vec<MessageEntry>, bool), ApplicationError> {
        let game_state = self.message_service.load_or_fresh();
        let entries: Vec<_> = game_state.narrative.history().to_vec();
        let has_last_trigger = game_state.narrative.last_trigger.is_some();
        Ok((entries, has_last_trigger))
    }

    /// The offered option set for the input-surface dock. The set is current
    /// game state — it survives a page reload and clears at every turn end
    /// (the pipeline rewrites it, never the UI).
    pub fn get_current_options(&self) -> Result<Vec<String>, ApplicationError> {
        let game_state = self.message_service.load_or_fresh();
        Ok(game_state.narrative.current_options.clone())
    }

    pub fn get_current_game_display_name(&self) -> Result<String, ApplicationError> {
        let storage = &self.storage;
        match storage.get_game(storage.current_game_id())? {
            Some(g) => Ok(g.display_name),
            None => Ok("Unknown".to_string()),
        }
    }

    pub fn list_latest_llm_messages(
        &self,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        self.storage
            .list_latest_llm_messages(limit)
            .map_err(Into::into)
    }

    pub fn role_health(&self) -> Result<Vec<RoleHealth>, ApplicationError> {
        // Retention keeps the whole table inside one window, so reading it
        // whole sees every role's newest attempt however long ago it ran.
        let messages = self
            .storage
            .list_latest_llm_messages(LLM_MESSAGE_RETENTION_LIMIT)?;
        let health = Role::ALL
            .iter()
            .map(|role| {
                let newest = messages
                    .iter()
                    .filter(|message| message.agent_name == role.agent_name())
                    .max_by_key(|message| (message.created_at, message.id));
                RoleHealth {
                    role: *role,
                    backend_model: newest.map(|m| format!("{} {}", m.backend_name, m.model_name)),
                    last_error: newest.and_then(|m| m.error_message.clone()),
                }
            })
            .collect();
        Ok(health)
    }

    pub fn active_quantifier_prompt(&self) -> String {
        let preset_id = self
            .storage
            .get_settings()
            .map(|s| self.storage.active_quantifier_preset_id(&s))
            .unwrap_or_default();
        match self.storage.get_preset(&preset_id) {
            Ok(Some(preset)) => preset.assemble_text(&[], None, None),
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
}
