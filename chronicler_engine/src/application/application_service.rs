//! Application service orchestrates game state mutations, persistence, and
//! game-service calls. It acts as the logic firewall between HTTP handlers
//! and the domain.
//!
//! [DOC: docs/architecture/system.md]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use serde::Serialize;

use crate::application::context::GameServiceContext;
use crate::application::game_service::GameService;
use crate::error::{EngineError, internal_error};
use crate::model::game::{Game, generate_game_name};
use crate::model::llm_message::LlmMessage;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogEntry};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::trigger::NpcEncounterState;

/// Unified error type for the application service layer.
/// Distinguishes validation failures (400), system errors (500),
/// and known runtime states like shutdown (503).
#[derive(Debug)]
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

impl From<ApplicationError> for EngineError {
    fn from(e: ApplicationError) -> Self {
        match e {
            ApplicationError::Engine(err) => err,
            ApplicationError::Validation(msg) => EngineError::Config(msg),
            ApplicationError::ShuttingDown => {
                EngineError::Config("Server is shutting down".to_string())
            }
            ApplicationError::ConcurrentGeneration => {
                EngineError::Config("Generation in progress".to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProcessActionResult {
    Started,
    ConcurrentGeneration,
    ShuttingDown,
}

#[derive(Serialize)]
pub struct DebugStateView {
    pub current_room_id: String,
    pub npcs_in_area: Vec<String>,
    pub generation_status: GenerationStatus,
    pub generation_phase: GenerationPhase,
    pub npc_encounter_log: HashMap<String, NpcEncounterState>,
    pub narration_history_tail: Vec<LogEntry>,
    pub narration_history_length: usize,
    pub dynamic_rooms: Vec<String>,
    pub dynamic_room_count: usize,
    pub last_error: Option<String>,
    pub quantifier_confidence: Option<String>,
    pub backend_name: Option<String>,
    pub model_name: Option<String>,
}

pub trait ApplicationService: Send + Sync {
    fn process_action(
        &self,
        ctx: GameServiceContext,
        input: String,
    ) -> Result<ProcessActionResult, EngineError>;
    fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn retrigger(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn reset(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn switch_swipe(
        &self,
        ctx: GameServiceContext,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError>;
    fn edit_history(
        &self,
        ctx: GameServiceContext,
        id: u64,
        text: String,
    ) -> Result<(), ApplicationError>;
    fn delete_last(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn create_game(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError>;
    fn switch_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError>;
    fn delete_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError>;
    fn list_games(&self, ctx: GameServiceContext) -> Result<Vec<Game>, ApplicationError>;
    fn current_game_id(&self, ctx: GameServiceContext) -> u64;
    fn get_generating_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError>;
    fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError>;
    fn get_current_game_name(&self, ctx: GameServiceContext) -> Result<String, ApplicationError>;
    fn list_latest_llm_messages(
        &self,
        ctx: GameServiceContext,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError>;
    fn get_story_log_entries(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(Vec<crate::model::state::LogEntry>, bool), ApplicationError>;
    fn get_input_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError>;
    fn get_current_room_view(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(String, Option<String>), ApplicationError>;
    fn get_npc_headshots(
        &self,
        ctx: GameServiceContext,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError>;
    fn get_debug_state(&self, ctx: GameServiceContext) -> Result<DebugStateView, ApplicationError>;
}

pub struct DefaultApplicationService {
    game_service: Arc<dyn GameService>,
}

impl DefaultApplicationService {
    pub fn new(game_service: Arc<dyn GameService>) -> Self {
        Self { game_service }
    }

    fn load_state(&self, ctx: &GameServiceContext) -> Result<GameState, ApplicationError> {
        crate::application::context::try_load_state(ctx).map_err(Into::into)
    }
}

impl ApplicationService for DefaultApplicationService {
    fn process_action(
        &self,
        ctx: GameServiceContext,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = self.load_state(&ctx)?;
        let player_name = game_state.player.sheet.name.clone();
        game_state.add_log(
            input.clone(),
            Some(player_name.clone()),
            crate::model::state::LogType::Input,
        );

        if ctx
            .is_generating
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(ProcessActionResult::ConcurrentGeneration);
        }

        game_state.narrative.input_buffer.status = GenerationStatus::Generating;
        game_state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        if let Err(e) =
            crate::application::game_service::save_message_and_snapshot(&ctx, &mut game_state)
        {
            ctx.is_generating.store(false, Ordering::SeqCst);
            return Err(e);
        }

        if ctx.cancel_token.is_cancelled() {
            let mut gs = self.load_state(&ctx)?;
            gs.narrative.input_buffer.status = GenerationStatus::Idle;
            let snapshot = GameStateSnapshot::from_game_state(&gs);
            if let Err(e) = ctx.snapshot_storage.save(&snapshot) {
                log::error!("Failed to save shutdown snapshot: {e}");
            }
            return Ok(ProcessActionResult::ShuttingDown);
        }

        // [DOC: docs/architecture/invariants.md#INV-004]
        let game_service = Arc::clone(&self.game_service);
        let ctx_clone = ctx.clone();
        tokio::task::spawn_blocking(move || {
            let _guard = GenerationGuard(Arc::clone(&ctx_clone.is_generating));
            if ctx_clone.cancel_token.is_cancelled() {
                return;
            }
            game_service.execute_action(ctx_clone, input, player_name);
        });

        Ok(ProcessActionResult::Started)
    }

    fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let mut game_state = self.load_state(&ctx)?;
        if game_state.narrative.history.last_input_text().is_none() {
            return Err(ApplicationError::validation("No input to retry"));
        }

        game_state.narrative.input_buffer.status = GenerationStatus::Generating;
        game_state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        let snapshot = GameStateSnapshot::from_game_state(&game_state);
        ctx.snapshot_storage.save(&snapshot)?;

        if ctx.cancel_token.is_cancelled() {
            return Err(ApplicationError::ShuttingDown);
        }

        // [DOC: docs/architecture/invariants.md#INV-004]
        let game_service = Arc::clone(&self.game_service);
        let ctx_clone = ctx.clone();
        tokio::task::spawn_blocking(move || {
            if ctx_clone.cancel_token.is_cancelled() {
                return;
            }
            game_service.retry_last_response(ctx_clone);
        });

        Ok(())
    }

    fn retrigger(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        if game_state.narrative.last_trigger.is_none() {
            return Err(ApplicationError::validation("No trigger context available"));
        }

        let messages = ctx.message_storage.load_messages()?;
        let Some(last_msg) = messages.last() else {
            return Err(ApplicationError::validation("No messages to retrigger"));
        };
        let is_narration = last_msg.log_type == crate::model::state::LogType::Narration
            || last_msg.log_type == crate::model::state::LogType::Dialogue;
        if !is_narration || last_msg.event_header.is_some() {
            return Err(ApplicationError::validation(
                "Last message must be a narration to retrigger",
            ));
        }

        let mut game_state = game_state;
        game_state.narrative.input_buffer.status = GenerationStatus::Generating;
        game_state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        let snapshot = GameStateSnapshot::from_game_state(&game_state);
        ctx.snapshot_storage.save(&snapshot)?;

        if ctx.cancel_token.is_cancelled() {
            return Err(ApplicationError::ShuttingDown);
        }

        // [DOC: docs/architecture/invariants.md#INV-004]
        let game_service = Arc::clone(&self.game_service);
        let ctx_clone = ctx.clone();
        tokio::task::spawn_blocking(move || {
            if ctx_clone.cancel_token.is_cancelled() {
                return;
            }
            game_service.retrigger_event(ctx_clone);
        });

        Ok(())
    }

    fn reset(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let current_id = ctx.snapshot_storage.current_game_id();
        let world_name = ctx.world.name.clone();

        ctx.snapshot_storage.delete_game(current_id)?;

        let existing_names: Vec<String> = ctx
            .snapshot_storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_name == world_name)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_name, &existing_names);
        let new_id = ctx.snapshot_storage.create_game(&world_name, &new_name)?;

        ctx.snapshot_storage.set_game_id(new_id);
        ctx.message_storage.set_game_id(new_id);

        let mut initial_state = build_fresh_initial_state(&ctx);
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = ctx.snapshot_storage.save(&snapshot)?;

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.id == 0 {
                msg.snapshot_id = Some(snapshot_id);
                let id = ctx.message_storage.insert_message(&*msg)?;
                msg.id = id;
            }
        }

        Ok(())
    }

    fn switch_swipe(
        &self,
        ctx: GameServiceContext,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let messages = ctx.message_storage.load_messages()?;
        let is_last = messages.last().map(|m| m.id == message_id).unwrap_or(false);
        if !is_last {
            return Err(ApplicationError::validation(
                "Only the last message can be swiped",
            ));
        }

        ctx.message_storage
            .update_active_swipe(message_id, swipe_index)?;

        let target_msg = messages
            .iter()
            .find(|m| m.id == message_id)
            .ok_or_else(|| app_err_internal("Message not found"))?;
        let target_swipe = target_msg
            .swipes
            .get(swipe_index)
            .ok_or_else(|| app_err_internal("Swipe index out of bounds"))?;
        let snapshot_id = target_swipe
            .snapshot_id
            .ok_or_else(|| app_err_internal("Swipe has no associated snapshot"))?;

        let mut snapshot = ctx
            .snapshot_storage
            .load_by_id(snapshot_id)?
            .ok_or_else(|| app_err_internal("Snapshot not found"))?;

        snapshot.created_at = Utc::now();
        ctx.snapshot_storage.save(&snapshot)?;

        Ok(())
    }

    fn edit_history(
        &self,
        ctx: GameServiceContext,
        id: u64,
        text: String,
    ) -> Result<(), ApplicationError> {
        let latest = ctx.snapshot_storage.load_latest()?;
        let mut guard = self.load_state(&ctx)?;
        guard.narrative.history.edit(id, text.clone())?;
        if latest.is_some() {
            let snapshot = GameStateSnapshot::from_game_state(&guard);
            ctx.snapshot_storage.save(&snapshot)?;
            ctx.message_storage.update_message(id, &text)?;
        }
        Ok(())
    }

    fn delete_last(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let mut guard = self.load_state(&ctx)?;
        let last_id = guard
            .narrative
            .history
            .last()
            .map(|m| m.id)
            .ok_or_else(|| {
                ApplicationError::Engine(EngineError::Internal(internal_error("History is empty")))
            })?;
        guard.narrative.history.delete_last()?;
        let snapshot = GameStateSnapshot::from_game_state(&guard);
        ctx.snapshot_storage.save(&snapshot)?;
        ctx.message_storage.delete_message(last_id)?;
        Ok(())
    }

    fn create_game(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let world_name = ctx.world.name.clone();
        let games = ctx.snapshot_storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);
        let new_id = ctx.snapshot_storage.create_game(&world_name, &name)?;

        let old_id = ctx.snapshot_storage.current_game_id();
        ctx.snapshot_storage.set_game_id(new_id);
        ctx.message_storage.set_game_id(new_id);

        let mut initial_state = build_fresh_initial_state(&ctx);
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = match ctx.snapshot_storage.save(&snapshot) {
            Ok(id) => id,
            Err(e) => {
                ctx.snapshot_storage.set_game_id(old_id);
                ctx.message_storage.set_game_id(old_id);
                return Err(e.into());
            }
        };

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.id == 0 {
                msg.snapshot_id = Some(snapshot_id);
                match ctx.message_storage.insert_message(&*msg) {
                    Ok(id) => msg.id = id,
                    Err(e) => log::error!("Create game failed: could not persist message: {e}"),
                }
            }
        }

        Ok(new_id)
    }

    fn switch_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        match ctx.snapshot_storage.get_game(id)? {
            Some(game) => {
                if game.world_name != ctx.world.name {
                    return Err(ApplicationError::validation(
                        "Game belongs to a different world",
                    ));
                }
            }
            None => return Err(ApplicationError::validation("Game not found")),
        }

        ctx.snapshot_storage.set_game_id(id);
        ctx.message_storage.set_game_id(id);

        Ok(())
    }

    fn delete_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if id == ctx.snapshot_storage.current_game_id() {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }

        ctx.snapshot_storage.delete_game(id)?;
        Ok(())
    }

    fn list_games(&self, ctx: GameServiceContext) -> Result<Vec<Game>, ApplicationError> {
        ctx.snapshot_storage.list_games().map_err(Into::into)
    }

    fn current_game_id(&self, ctx: GameServiceContext) -> u64 {
        ctx.snapshot_storage.current_game_id()
    }

    fn get_generating_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        Ok((
            game_state.narrative.input_buffer.status.clone(),
            game_state.narrative.input_buffer.phase.clone(),
        ))
    }

    fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let mut game_state = self.load_state(&ctx)?;
        game_state.narrative.input_buffer.status = GenerationStatus::Idle;
        let snapshot = GameStateSnapshot::from_game_state(&game_state);
        ctx.snapshot_storage.save(&snapshot)?;
        Ok(())
    }

    fn get_current_game_name(&self, ctx: GameServiceContext) -> Result<String, ApplicationError> {
        match ctx
            .snapshot_storage
            .get_game(ctx.snapshot_storage.current_game_id())?
        {
            Some(g) => Ok(g.name),
            None => Ok("Unknown".to_string()),
        }
    }

    fn list_latest_llm_messages(
        &self,
        ctx: GameServiceContext,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        ctx.llm_message_storage
            .list_latest(limit)
            .map_err(Into::into)
    }

    fn get_story_log_entries(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(Vec<crate::model::state::LogEntry>, bool), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        let entries: Vec<_> = game_state.narrative.history().to_vec();
        let has_last_trigger = game_state.narrative.last_trigger.is_some();
        Ok((entries, has_last_trigger))
    }

    fn get_input_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        Ok((
            game_state.narrative.input_buffer.status.clone(),
            game_state.narrative.input_buffer.phase.clone(),
        ))
    }

    fn get_current_room_view(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(String, Option<String>), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        let room = game_state
            .current_room()
            .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;
        let image_path = room
            .image_path
            .clone()
            .or_else(|| game_state.world.default_room_image.clone());
        Ok((room.name.clone(), image_path))
    }

    fn get_npc_headshots(
        &self,
        ctx: GameServiceContext,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        let npc_ids: Vec<String> = if scene_only {
            game_state
                .scene
                .npcs_in_area
                .iter()
                .map(|npc| npc.id.clone())
                .collect()
        } else {
            game_state.npcs.keys().cloned().collect()
        };
        let npc_data: Vec<(String, String)> = npc_ids
            .iter()
            .filter_map(|id| {
                let npc = game_state.npcs.get(id)?;
                let image_path = npc.sheet.preferred_image()?.to_string();
                let name = npc.sheet.name.clone();
                Some((image_path, name))
            })
            .collect();
        Ok(npc_data)
    }

    fn get_debug_state(&self, ctx: GameServiceContext) -> Result<DebugStateView, ApplicationError> {
        let game_state = self.load_state(&ctx)?;

        let history_tail: Vec<LogEntry> = game_state
            .narrative
            .history()
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect();

        let npcs_in_area: Vec<String> = game_state
            .scene
            .npcs_in_area
            .iter()
            .map(|npc| npc.id.clone())
            .collect();

        let dynamic_rooms: Vec<String> =
            game_state.movement.dynamic_rooms.keys().cloned().collect();

        let last_error = match &game_state.narrative.input_buffer.status {
            GenerationStatus::Error(msg) => Some(msg.clone()),
            _ => None,
        };

        Ok(DebugStateView {
            current_room_id: game_state.movement.current_room_id.clone(),
            npcs_in_area,
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
        })
    }
}

// [DOC: docs/architecture/invariants.md#INV-004]
/// RAII guard that clears the `is_generating` flag on drop.
struct GenerationGuard(Arc<AtomicBool>);

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn app_err_internal(msg: impl Into<String>) -> ApplicationError {
    ApplicationError::Engine(EngineError::Internal(internal_error(msg)))
}

fn build_fresh_initial_state(ctx: &GameServiceContext) -> GameState {
    let mut initial_state = GameState::new(
        Arc::clone(&ctx.world),
        Arc::clone(&ctx.map),
        Arc::clone(&ctx.player),
        (*ctx.npcs).values().cloned().collect(),
        ctx.world.starting_room_id.clone(),
    );

    if let Some(scenario) = ctx.world.default_scenario() {
        let room_name = crate::engine::logic::find_room_in_world_map(
            &initial_state,
            &ctx.world.starting_room_id,
        )
        .map(|r| r.name.clone())
        .unwrap_or_else(|| ctx.world.starting_room_id.clone());

        initial_state.narrative.pending_location = Some(room_name);
        let text = scenario.text.replace("{{user}}", &ctx.player.sheet.name);
        if !text.is_empty() {
            initial_state.add_log(text, None, crate::model::state::LogType::Narration);
        }

        initial_state.init_scenario_npcs(scenario);
    }

    initial_state
}
