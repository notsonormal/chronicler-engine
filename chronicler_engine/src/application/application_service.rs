//! [DOC: docs/system/game_flow.md]
//! Main application service coordinating game operations
//! arch-lint: storage-direct — intentional, see ADR-027

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::application::action_pipeline::execute_action_impl;
use crate::application::game_service::GameService;
use crate::application::generation_guard::GenerationGuard;

use crate::error::{EngineError, LlmFailure};
use crate::domain::model::character::NpcCard;
use crate::domain::model::game::{Game, generate_game_name};
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;

use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::template::{render_template, TemplateVars};
use crate::domain::model::trigger::NpcEncounterState;
use crate::domain::model::world::WorldCard;
use crate::domain::model::map::MapDef;
use crate::application::narrative_prompt::assembler::assemble_prompt_text;
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::worlds::WorldWithMap;

#[derive(Clone)]
pub struct WorldSnapshot {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<crate::domain::model::character::PlayerCard>,
    pub npcs: Arc<std::collections::HashMap<String, NpcCard>>,
}

impl WorldSnapshot {
    pub fn empty() -> Self {
        Self {
            world: Arc::new(WorldCard::default()),
            map: Arc::new(MapDef::default()),
            player: Arc::new(crate::domain::model::character::PlayerCard::default()),
            npcs: Arc::new(std::collections::HashMap::new()),
        }
    }
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
    pub(crate) preset_storage: Arc<Storage>,
    pub(crate) settings: Arc<RwLock<AppSettings>>,
    pub(crate) cancel_token: CancellationToken,
    pub(crate) is_generating: Arc<AtomicBool>,
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
        Self {
            storage,
            preset_storage,
            settings,
            cancel_token,
            is_generating,
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
        &self.is_generating
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
        let mut npcs: std::collections::HashMap<String, crate::domain::model::character::NpcCard> =
            std::collections::HashMap::new();
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
        load_messages_with_swipes(&self.storage)
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

    pub fn process_action(&self, input: String) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = self.load_or_fresh()?;

        self.heal_stale_generating(&mut game_state);

        let player_name = game_state.player.sheet.name.clone();
        if !input.is_empty() {
            game_state.add_message(input.clone(), Some(player_name.clone()), MessageType::Input);
        }

        match self.claim_generation_slot(&mut game_state, &player_name)? {
            ProcessActionResult::ConcurrentGeneration => {
                return Ok(ProcessActionResult::ConcurrentGeneration);
            }
            ProcessActionResult::Started => {}
            ProcessActionResult::ShuttingDown => unreachable!(),
        }

        if self.cancel_token.is_cancelled() {
            if let Ok(mut gs) = self.load_or_fresh() {
                gs.narrative.input_buffer.status = GenerationStatus::Idle;
                let snapshot = GameStateSnapshot::from_game_state(&gs);
                if let Err(e) = self.storage.save_snapshot(&snapshot) {
                    tracing::error!("Failed to save shutdown snapshot: {e}");
                }
            }
            self.release_generation_slot();
            return Ok(ProcessActionResult::ShuttingDown);
        }

        let is_generating = Arc::clone(&self.is_generating);
        crate::application::spawn_pipeline_task(Arc::new(self.clone()), move |app| {
            tracing::debug!("spawn_blocking: task started");
            let _guard = GenerationGuard(Arc::clone(&is_generating));
            if app.cancel_token.is_cancelled() {
                tracing::debug!("spawn_blocking: cancelled before execute_action");
                return;
            }
            execute_action_impl(app, input);
            tracing::debug!("spawn_blocking: execute_action completed");
        });
        Ok(ProcessActionResult::Started)
    }

    /// Heal stale Generating status when no active generation is running.
    /// If `is_generating` AtomicBool is false but persisted status reports
    /// `Generating`, reset status+phase to Idle.
    fn heal_stale_generating(&self, state: &mut GameState) {
        if !self.is_generating.load(Ordering::SeqCst)
            && state.narrative.input_buffer.status.is_generating()
        {
            tracing::warn!(
                "Found stale Generating status without active generation, resetting to Idle"
            );
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
        }
    }

    /// Try to claim the generation slot via CAS on the AtomicBool, then persist
    /// the Generating status. Outcomes:
    /// - `(Ok Started)` CAS won and save succeeded; AtomicBool=true, status=Generating.
    /// - `(Ok ConcurrentGeneration)` CAS lost; caller should return without rollback.
    /// - `(Err ...)` CAS won but save failed; AtomicBool still true — caller MUST call
    ///   `release_generation_slot` to roll back, then propagate the error.
    fn claim_generation_slot(
        &self,
        state: &mut GameState,
        _player_name: &str,
    ) -> Result<ProcessActionResult, EngineError> {
        if self
            .is_generating
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(ProcessActionResult::ConcurrentGeneration);
        }

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        if let Err(e) = self.save_message_and_snapshot(state) {
            tracing::debug!("claim_generation_slot: save failed; caller must release slot");
            return Err(e);
        }
        tracing::debug!("process_action: state saved, spawning blocking task");
        Ok(ProcessActionResult::Started)
    }

    /// Release the generation slot by clearing the AtomicBool. Caller must invoke
    /// this on any error path after CAS succeeded.
    fn release_generation_slot(&self) {
        self.is_generating.store(false, Ordering::SeqCst);
    }

    pub fn continue_narration(&self) -> Result<ProcessActionResult, EngineError> {
        self.process_action(String::new())
    }

    pub fn create_game(&self, world_key: &str, persona_key: &str) -> Result<u64, ApplicationError> {
        if self.is_generating.load(Ordering::SeqCst) {
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
        if self.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if self.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        self.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, id: u64) -> Result<(), ApplicationError> {
        if self.is_generating.load(Ordering::SeqCst) {
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
