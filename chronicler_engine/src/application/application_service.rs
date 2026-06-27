//! [DOC: docs/system/game_flow.md]
//! Main application service coordinating game operations

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::application::action_pipeline::execute_action_impl;
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::application::game_service::GameService;
use crate::application::message_editing::MessageEditingService;
use crate::bootstrap::build_fresh_initial_state;
use crate::error::EngineError;
use crate::model::game::{Game, generate_game_name};
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

pub struct DefaultApplicationService {
    game_service: Arc<GameService>,
    editing: MessageEditingService,
}

impl DefaultApplicationService {
    pub fn new(game_service: Arc<GameService>) -> Self {
        Self {
            editing: MessageEditingService::new(Arc::clone(&game_service)),
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

        self.spawn_pipeline_task(ctx, input);
        Ok(ProcessActionResult::Started)
    }

    fn spawn_pipeline_task(&self, ctx: GameServiceContext, input: String) {
        let game_service = Arc::clone(&self.game_service);
        tokio::task::spawn_blocking(move || {
            tracing::debug!("spawn_blocking: task started");
            let _guard = GenerationGuard(Arc::clone(&ctx.is_generating));
            if ctx.cancel_token.is_cancelled() {
                tracing::debug!("spawn_blocking: cancelled before execute_action");
                return;
            }
            execute_action_impl(&*game_service, ctx.clone(), input);
            tracing::debug!("spawn_blocking: execute_action completed");
        });
    }

    pub fn continue_narration(
        &self,
        ctx: GameServiceContext,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action(ctx, String::new())
    }

    pub fn create_game(&self, ctx: GameServiceContext) -> Result<u64, ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let world_with_map = ctx
            .storage
            .get_world(&ctx.world.key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let games = ctx.storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);

        let new_id = ctx.storage.create_game(
            &world_name,
            &ctx.world.key,
            &ctx.player.key,
            &ctx.player.sheet.name,
            &name,
        )?;
        let old_id = ctx.storage.current_game_id();
        ctx.set_game_id(new_id);

        match Self::persist_initial_state_with_swipes(&ctx) {
            Ok(_) => {}
            Err(e) => {
                ctx.set_game_id(old_id);
                return Err(e);
            }
        }

        Ok(new_id)
    }

    pub fn switch_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if ctx.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        ctx.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, ctx: GameServiceContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if id == ctx.storage.current_game_id() {
            return Err(ApplicationError::validation(
                "Cannot delete the active game",
            ));
        }
        ctx.storage.delete_game(id)?;
        Ok(())
    }

    pub fn list_games(&self, ctx: GameServiceContext) -> Result<Vec<Game>, ApplicationError> {
        ctx.storage.list_games().map_err(Into::into)
    }

    pub fn current_game_id(&self, ctx: GameServiceContext) -> u64 {
        ctx.storage.current_game_id()
    }

    pub fn reset(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let current_id = ctx.storage.current_game_id();
        let world_key = ctx.world.key.clone();
        let world_name = ctx.world.name.clone();

        ctx.storage.delete_game(current_id)?;

        let existing_names: Vec<String> = ctx
            .storage
            .list_games()?
            .into_iter()
            .filter(|g| g.world_key == world_key)
            .map(|g| g.name)
            .collect();

        let new_name = generate_game_name(&world_name, &existing_names);
        let new_id = ctx.storage.create_game(
            &world_name,
            &world_key,
            &ctx.player.key,
            &ctx.player.sheet.name,
            &new_name,
        )?;
        ctx.set_game_id(new_id);

        // snapshot already committed; message/swipe failures logged, not propagated
        let _ = Self::persist_initial_state_with_swipes(&ctx);

        Ok(())
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
        super::query_handlers::get_generating_status(ctx)
    }

    pub fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        super::query_handlers::reset_generating_status(ctx)
    }

    pub fn get_current_game_name(
        &self,
        ctx: GameServiceContext,
    ) -> Result<String, ApplicationError> {
        super::query_handlers::get_current_game_name(ctx)
    }

    pub fn list_latest_llm_messages(
        &self,
        ctx: GameServiceContext,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        super::query_handlers::list_latest_llm_messages(ctx, limit)
    }

    pub fn get_story_log_entries(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(Vec<MessageEntry>, bool), ApplicationError> {
        super::query_handlers::get_story_log_entries(ctx)
    }

    pub fn get_input_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
        super::query_handlers::get_input_status(ctx)
    }

    pub fn get_current_room_view(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(String, Option<String>), ApplicationError> {
        super::query_handlers::get_current_room_view(ctx)
    }

    pub fn get_npc_headshots(
        &self,
        ctx: GameServiceContext,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        super::query_handlers::get_npc_headshots(ctx, scene_only)
    }

    pub fn get_debug_state(
        &self,
        ctx: GameServiceContext,
    ) -> Result<DebugStateView, ApplicationError> {
        super::query_handlers::get_debug_state(ctx)
    }

    pub fn list_worlds(&self, ctx: GameServiceContext) -> Result<Vec<WorldCard>, ApplicationError> {
        ctx.storage.list_worlds().map_err(Into::into)
    }

    pub fn get_world(
        &self,
        ctx: GameServiceContext,
        key: &str,
    ) -> Result<Option<WorldWithMap>, ApplicationError> {
        ctx.storage.get_world(key).map_err(Into::into)
    }

    pub fn create_world(
        &self,
        ctx: GameServiceContext,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        ctx.storage
            .create_world(&world_card, &map)
            .map_err(Into::into)
    }

    pub fn update_world(
        &self,
        ctx: GameServiceContext,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        ctx.storage
            .update_world(id, &world_card, &map)
            .map_err(Into::into)
    }

    pub fn delete_world(&self, ctx: GameServiceContext, key: &str) -> Result<(), ApplicationError> {
        ctx.storage.delete_world(key).map_err(Into::into)
    }

    pub fn list_personas(
        &self,
        ctx: GameServiceContext,
    ) -> Result<Vec<PlayerCard>, ApplicationError> {
        ctx.storage.list_personas().map_err(Into::into)
    }

    /// Build fresh initial state, persist snapshot, then persist the unpersisted
    /// trailing message and its swipes. Snapshot save failures propagate;
    /// message/swipe persistence failures are logged and swallowed because the
    /// snapshot is already committed by the time those run.
    fn persist_initial_state_with_swipes(
        ctx: &GameServiceContext,
    ) -> Result<u64, ApplicationError> {
        let mut initial_state = build_fresh_initial_state(ctx);
        let snapshot = GameStateSnapshot::from_game_state(&initial_state);
        let snapshot_id = ctx.storage.save_snapshot(&snapshot)?;

        if let Some(msg) = initial_state.narrative.history.last_mut() {
            if msg.is_unpersisted() {
                msg.set_snapshot_id(Some(snapshot_id));
                match ctx.storage.insert_message(&*msg) {
                    Ok(id) => {
                        msg.id = id;
                        for (index, swipe) in msg.swipes.iter().enumerate() {
                            if let Err(e) = ctx.storage.insert_swipe(id, swipe, index) {
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

struct GenerationGuard(Arc<AtomicBool>);

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
