//! [DOC: docs/system/game_flow.md]
//! Main application service coordinating game operations
//! arch-lint: storage-direct — intentional, see ADR-027

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::application::game_service::GameService;
use crate::application::generation_gate::GenerationGate;
use crate::application::persistence_gate::PersistenceGate;

use crate::error::{EngineError, LlmFailure};
use crate::domain::model::game::{Game, generate_game_name};
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;

use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageEntry;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::trigger::NpcEncounterState;
use crate::domain::model::world::WorldCard;
use crate::domain::model::map::MapDef;
use crate::application::narrative_prompt::assembler::assemble_prompt_text;
use crate::adapters::driven::storage::{PresetStore, Storage};
use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::application::persistence_gate::WorldSnapshot;

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

pub fn load_messages_with_swipes(storage: &Storage) -> Result<Vec<Message>, EngineError> {
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

pub enum ApplicationError {
    Validation(String),
    Engine(EngineError),
    ShuttingDown,
    ConcurrentGeneration,
}

impl ApplicationError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn is_user_displayable(&self) -> bool {
        matches!(
            self,
            Self::Validation(_) | Self::Engine(EngineError::WorldHasGames { .. })
        )
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "{msg}"),
            Self::Engine(e) => write!(f, "{e}"),
            Self::ShuttingDown => write!(f, "Server is shutting down"),
            Self::ConcurrentGeneration => write!(f, "Generation in progress"),
        }
    }
}

impl std::fmt::Debug for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Engine(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EngineError> for ApplicationError {
    fn from(e: EngineError) -> Self {
        Self::Engine(e)
    }
}

pub enum ProcessActionResult {
    Started,
    ConcurrentGeneration,
    ShuttingDown,
}

#[derive(Clone, Serialize)]
pub struct DebugStateView {
    pub current_room_id: String,
    pub npcs_in_area: Vec<String>,
    pub generation_status: GenerationStatus,
    pub generation_phase: GenerationPhase,
    pub npc_encounter_log: HashMap<String, NpcEncounterState>,
    pub narration_history_tail: Vec<MessageEntry>,
    pub narration_history_length: usize,
    pub dynamic_rooms: Vec<String>,
    pub dynamic_room_count: usize,
    pub last_error: Option<String>,
    pub quantifier_confidence: Option<String>,
    pub backend_name: Option<String>,
    pub model_name: Option<String>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct DefaultApplicationService {
    pub(crate) storage: Arc<Storage>,
    pub(crate) preset_storage: Arc<PresetStore>,
    pub(crate) persistence_gate: Arc<PersistenceGate>,
    pub(crate) settings: Arc<RwLock<AppSettings>>,
    pub(crate) generation_gate: GenerationGate,
    pub(crate) game_service: Arc<GameService>,
}

impl DefaultApplicationService {
    pub fn new(
        storage: Arc<Storage>,
        preset_storage: Arc<Storage>,
        settings: Arc<RwLock<AppSettings>>,
        cancel_token: CancellationToken,
        is_generating: Arc<AtomicBool>,
        game_service: Arc<GameService>,
    ) -> Self {
        let preset_storage = Arc::new(PresetStore::new(preset_storage));
        let persistence_gate = Arc::new(PersistenceGate::new(
            Arc::clone(&storage),
            Arc::clone(&preset_storage),
        ));
        let generation_gate = GenerationGate::new(cancel_token, is_generating);
        Self {
            storage,
            preset_storage,
            persistence_gate,
            settings,
            generation_gate,
            game_service,
        }
    }

    pub fn game_service(&self) -> &Arc<GameService> {
        &self.game_service
    }

    pub fn storage(&self) -> &Arc<Storage> {
        &self.storage
    }

    pub fn is_generating(&self) -> &Arc<AtomicBool> {
        self.generation_gate.is_generating()
    }

    pub fn cancel_token(&self) -> &CancellationToken {
        self.generation_gate.cancel_token()
    }

    pub fn settings(&self) -> &Arc<RwLock<AppSettings>> {
        &self.settings
    }

    pub fn preset_storage(&self) -> &Arc<PresetStore> {
        &self.preset_storage
    }

    pub(crate) fn load_world_snapshot(&self) -> Result<WorldSnapshot, EngineError> {
        self.persistence_gate.load_world_snapshot()
    }

    pub fn load_or_fresh(&self) -> Result<GameState, EngineError> {
        self.persistence_gate.load_or_fresh()
    }

    pub fn load_expecting_valid_state(&self) -> Result<GameState, EngineError> {
        self.persistence_gate.load_expecting_valid_state()
    }

    pub fn save_state(&self, state: &GameState) -> Result<u64, EngineError> {
        self.persistence_gate.save_state(state)
    }

    pub fn save_message_and_snapshot(&self, state: &mut GameState) -> Result<u64, EngineError> {
        self.persistence_gate.save_message_and_snapshot(state)
    }

    pub fn delete_and_remove_message(
        &self,
        state: &mut GameState,
        id: u64,
    ) -> Result<(), EngineError> {
        self.persistence_gate.delete_and_remove_message(state, id)
    }

    pub fn load_messages_with_swipes(&self) -> Result<Vec<Message>, EngineError> {
        self.persistence_gate.load_messages_with_swipes()
    }

    pub fn load_messages_into_state(&self, state: &mut GameState) {
        self.persistence_gate.load_messages_into_state(state)
    }

    pub fn build_fresh_initial_state(&self) -> Result<GameState, EngineError> {
        self.persistence_gate.build_fresh_initial_state()
    }

    pub fn load_messages(&self) -> Result<Vec<Message>, EngineError> {
        self.persistence_gate.load_messages()
    }

    pub fn update_message_text(&self, id: u64, text: &str) -> Result<(), EngineError> {
        self.persistence_gate.update_message_text(id, text)
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

    pub fn find_retry_anchor<'a>(
        &self,
        messages: &'a [Message],
    ) -> Option<(usize, &'a Message, u64)> {
        self.persistence_gate.find_retry_anchor(messages)
    }

    pub fn set_game_id(&self, game_id: u64) {
        self.persistence_gate.set_game_id(game_id);
    }

    pub fn process_action(&self, input: String) -> Result<ProcessActionResult, EngineError> {
        self.generation_gate.start_action(self, input)
    }

    #[allow(dead_code)]
    pub(crate) fn heal_stale_generating(&self, state: &mut GameState) {
        self.generation_gate.heal_stale_generating(self, state)
    }

    // On save failure after CAS wins, the AtomicBool is rolled back inside the gate.
    #[allow(dead_code)]
    pub(crate) fn claim_generation_slot(
        &self,
        state: &mut GameState,
    ) -> Result<ProcessActionResult, EngineError> {
        self.generation_gate.claim_generation_slot(self, state)
    }

    #[allow(dead_code)]
    pub(crate) fn release_generation_slot(&self) {
        self.generation_gate.release_generation_slot()
    }

    pub fn continue_narration(&self) -> Result<ProcessActionResult, EngineError> {
        self.process_action(String::new())
    }

    pub fn create_game(&self, world_key: &str, persona_key: &str) -> Result<u64, ApplicationError> {
        if self.is_generating().load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let world_with_map = self
            .storage
            .get_world(world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let player = self
            .storage
            .get_persona(persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;
        let games = self.storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);

        let new_id = self.storage.create_game(
            &world_name,
            world_key,
            persona_key,
            &player.sheet.name,
            &name,
        )?;
        let old_id = self.storage.current_game_id();
        self.set_game_id(new_id);

        match self.persist_initial_state_with_swipes() {
            Ok(_) => {}
            Err(e) => {
                self.set_game_id(old_id);
                return Err(e);
            }
        }

        Ok(new_id)
    }

    pub fn switch_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.is_generating().load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if self.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        self.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.is_generating().load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if id == self.storage.current_game_id() {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }
        self.storage.delete_game(id)?;
        Ok(())
    }

    pub fn list_games(&self) -> Result<Vec<Game>, ApplicationError> {
        self.storage.list_games().map_err(Into::into)
    }

    pub fn current_game_id(&self) -> u64 {
        self.storage.current_game_id()
    }

    pub fn reset(&self) -> Result<(), ApplicationError> {
        let current_id = self.storage.current_game_id();
        let game = self
            .storage
            .get_game(current_id)?
            .ok_or_else(|| ApplicationError::validation("Current game not found"))?;
        let world_key = game.world_key.clone();
        let persona_key = game.persona_key.clone();

        let world_with_map = self
            .storage
            .get_world(&world_key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let player = self
            .storage
            .get_persona(&persona_key)?
            .ok_or_else(|| ApplicationError::validation("Persona not found"))?;

        self.storage.delete_game(current_id)?;

        let existing_names: Vec<String> = self
            .storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_key == world_key)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_name, &existing_names);
        let new_id = self.storage.create_game(
            &world_name,
            &world_key,
            &persona_key,
            &player.sheet.name,
            &new_name,
        )?;
        self.set_game_id(new_id);

        let _ = self.persist_initial_state_with_swipes();

        Ok(())
    }

    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, ApplicationError> {
        self.storage.list_worlds().map_err(Into::into)
    }

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, ApplicationError> {
        self.storage.get_world(key).map_err(Into::into)
    }

    pub fn create_world(
        &self,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.storage
            .create_world(&world_card, &map)
            .map_err(Into::into)
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        self.storage
            .update_world(id, &world_card, &map)
            .map_err(Into::into)
    }

    pub fn delete_world(&self, key: &str) -> Result<(), ApplicationError> {
        self.storage.delete_world(key).map_err(Into::into)
    }

    pub fn list_personas(
        &self,
    ) -> Result<Vec<crate::domain::model::character::PlayerCard>, ApplicationError> {
        self.storage.list_personas().map_err(Into::into)
    }

    fn persist_initial_state_with_swipes(&self) -> Result<u64, ApplicationError> {
        let mut initial_state = self.build_fresh_initial_state()?;
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = self.storage.save_snapshot(&snapshot)?;

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                match self.storage.insert_message(&*msg) {
                    Ok(id) => {
                        msg.id = id;
                        for (index, swipe) in msg.swipes.iter().enumerate() {
                            if let Err(e) = self.storage.insert_swipe(id, swipe, index) {
                                tracing::error!("persist_initial_state: swipe {index} failed: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("persist_initial_state: message insert failed: {e}");
                    }
                }
            }
        }

        Ok(snapshot_id)
    }
}
