//! [DOC: docs/system/game_flow.md]
//! Main application service coordinating game operations

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::application::action_pipeline::execute_action_impl;
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::application::game_lifecycle::GameLifecycleService;
use crate::application::game_service::GameService;
use crate::application::message_editing::MessageEditingService;
use crate::application::query_handlers::QueryHandlers;
use crate::error::EngineError;
use crate::model::game::Game;
use crate::model::llm_message::LlmMessage;
use crate::model::state::{GenerationPhase, GenerationStatus, MessageEntry};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::trigger::NpcEncounterState;
use crate::model::world::WorldCard;
use crate::model::map::MapDef;
use crate::model::character::PlayerCard;
use crate::storage::worlds::WorldWithMap;

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

pub struct DefaultApplicationService {
    game_service: Arc<GameService>,
    lifecycle: GameLifecycleService,
    editing: MessageEditingService,
    queries: QueryHandlers,
}

impl DefaultApplicationService {
    pub fn new(game_service: Arc<GameService>) -> Self {
        Self {
            lifecycle: GameLifecycleService::new(),
            editing: MessageEditingService::new(Arc::clone(&game_service)),
            queries: QueryHandlers::new(),
            game_service,
        }
    }

    pub fn game_service(&self) -> &Arc<GameService> {
        &self.game_service
    }

    pub fn process_action(
        &self,
        ctx: GameServiceContext,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = load_or_fresh(&ctx);

        // Self-heal: if a previous generation panicked, is_generating was cleared
        // by GenerationGuard::Drop but the persisted status may still be Generating.
        // Reset to Idle so the new request can proceed normally.
        if !ctx.is_generating.load(Ordering::SeqCst)
            && game_state.narrative.input_buffer.status.is_generating()
        {
            tracing::warn!(
                "Found stale Generating status without active generation, resetting to Idle"
            );
            game_state.narrative.input_buffer.status = GenerationStatus::Idle;
            game_state.narrative.input_buffer.phase = GenerationPhase::default();
        }

        let player_name = game_state.player.sheet.name.clone();

        if !input.is_empty() {
            game_state.add_message(
                input.clone(),
                Some(player_name.clone()),
                crate::model::state::MessageType::Input,
            );
        }

        if ctx
            .is_generating
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(ProcessActionResult::ConcurrentGeneration);
        }

        game_state.narrative.input_buffer.status = GenerationStatus::Generating;
        game_state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        if let Err(e) = crate::application::save_message_and_snapshot(&ctx, &mut game_state) {
            tracing::debug!(
                "process_action: save failed, setting is_generating=false and returning error"
            );
            ctx.is_generating.store(false, Ordering::SeqCst);
            return Err(e);
        }
        tracing::debug!("process_action: state saved, spawning blocking task");

        if ctx.cancel_token.is_cancelled() {
            let mut gs = load_or_fresh(&ctx);
            gs.narrative.input_buffer.status = GenerationStatus::Idle;
            let snapshot = GameStateSnapshot::from_game_state(&gs);
            if let Err(e) = ctx.storage.save_snapshot(&snapshot) {
                tracing::error!("Failed to save shutdown snapshot: {e}");
            }
            return Ok(ProcessActionResult::ShuttingDown);
        }
        tracing::debug!("process_action: spawning blocking task");
        let game_service = Arc::clone(&self.game_service);
        let ctx_clone = ctx.clone();
        tokio::task::spawn_blocking(move || {
            tracing::debug!("spawn_blocking: task started");
            let _guard = GenerationGuard(Arc::clone(&ctx_clone.is_generating));
            if ctx_clone.cancel_token.is_cancelled() {
                tracing::debug!("spawn_blocking: cancelled before execute_action");
                return;
            }
            execute_action_impl(&*game_service, ctx_clone.clone(), input, player_name);
            tracing::debug!("spawn_blocking: execute_action completed");
        });
        Ok(ProcessActionResult::Started)
    }

    pub fn continue_narration(
        &self,
        ctx: GameServiceContext,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action(ctx, String::new())
    }

    pub fn create_game(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError> {
        self.lifecycle.create_game(ctx)
    }

    pub fn switch_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        self.lifecycle.switch_game(ctx, id)
    }

    pub fn delete_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        self.lifecycle.delete_game(ctx, id)
    }

    pub fn list_games(&self, ctx: GameServiceContext) -> Result<Vec<Game>, ApplicationError> {
        self.lifecycle.list_games(ctx)
    }

    pub fn current_game_id(&self, ctx: GameServiceContext) -> u64 {
        self.lifecycle.current_game_id(ctx)
    }

    pub fn reset(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        self.lifecycle.reset(ctx)
    }

    pub fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        self.editing.retry(ctx)
    }

    pub fn retrigger(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        self.editing.retrigger(ctx)
    }

    pub fn switch_swipe(
        &self,
        ctx: GameServiceContext,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError> {
        self.editing.switch_swipe(ctx, message_id, swipe_index)
    }

    pub fn edit_history(
        &self,
        ctx: GameServiceContext,
        id: u64,
        text: String,
    ) -> Result<(), ApplicationError> {
        self.editing.edit_history(ctx, id, text)
    }

    pub fn delete_last(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        self.editing.delete_last(ctx)
    }

    pub fn get_generating_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
        self.queries.get_generating_status(ctx)
    }

    pub fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        self.queries.reset_generating_status(ctx)
    }

    pub fn get_current_game_name(
        &self,
        ctx: GameServiceContext,
    ) -> Result<String, ApplicationError> {
        self.queries.get_current_game_name(ctx)
    }

    pub fn list_latest_llm_messages(
        &self,
        ctx: GameServiceContext,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        self.queries.list_latest_llm_messages(ctx, limit)
    }

    pub fn get_story_log_entries(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(Vec<MessageEntry>, bool), ApplicationError> {
        self.queries.get_story_log_entries(ctx)
    }

    pub fn get_input_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
        self.queries.get_input_status(ctx)
    }

    pub fn get_current_room_view(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(String, Option<String>), ApplicationError> {
        self.queries.get_current_room_view(ctx)
    }

    pub fn get_npc_headshots(
        &self,
        ctx: GameServiceContext,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        self.queries.get_npc_headshots(ctx, scene_only)
    }

    pub fn get_debug_state(
        &self,
        ctx: GameServiceContext,
    ) -> Result<DebugStateView, ApplicationError> {
        self.queries.get_debug_state(ctx)
    }

    // TODO(#tech-debt): Worlds CRUD methods are pure passthroughs to GameLifecycleService.
    // Combined with lifecycle layer, this creates 14 identity wrappers for zero logic.
    pub fn list_worlds(&self, ctx: GameServiceContext) -> Result<Vec<WorldCard>, ApplicationError> {
        self.lifecycle.list_worlds(ctx)
    }

    pub fn get_world(
        &self,
        ctx: GameServiceContext,
        key: &str,
    ) -> Result<Option<WorldWithMap>, ApplicationError> {
        self.lifecycle.get_world(ctx, key)
    }

    pub fn create_world(
        &self,
        ctx: GameServiceContext,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.lifecycle.create_world(ctx, world_card, map)
    }

    pub fn update_world(
        &self,
        ctx: GameServiceContext,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        self.lifecycle.update_world(ctx, id, world_card, map)
    }

    pub fn delete_world(&self, ctx: GameServiceContext, key: &str) -> Result<(), ApplicationError> {
        self.lifecycle.delete_world(ctx, key)
    }

    pub fn list_personas(
        &self,
        ctx: GameServiceContext,
    ) -> Result<Vec<PlayerCard>, ApplicationError> {
        self.lifecycle.list_personas(ctx)
    }
}

struct GenerationGuard(Arc<AtomicBool>);

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
