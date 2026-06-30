//! [DOC: docs/system/game_flow.md]
//! Main application service coordinating game operations

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::application::action_pipeline::execute_action_impl;
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::application::game_service::GameService;
use crate::application::text_check_service::TextCheckService;

use crate::bootstrap::build_fresh_initial_state;
use crate::error::EngineError;
use crate::domain::model::game::{Game, generate_game_name};

use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::adapters::driven::storage::snapshot_blob::GameStateSnapshot;
use crate::domain::model::trigger::NpcEncounterState;
use crate::domain::model::world::WorldCard;
use crate::domain::model::map::MapDef;
use crate::adapters::driven::storage::worlds::WorldWithMap;

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
    text_check_service: Option<TextCheckService>,
}

impl DefaultApplicationService {
    pub fn new(game_service: Arc<GameService>) -> Self {
        Self {
            game_service,
            text_check_service: None,
        }
    }

    pub fn with_text_check_service(mut self, text_check_service: TextCheckService) -> Self {
        self.text_check_service = Some(text_check_service);
        self
    }

    pub fn game_service(&self) -> &Arc<GameService> {
        &self.game_service
    }

    pub fn text_check_service(&self) -> Option<&TextCheckService> {
        self.text_check_service.as_ref()
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
            game_state.add_message(input.clone(), Some(player_name.clone()), MessageType::Input);
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

        let is_generating = Arc::clone(&ctx.is_generating);
        crate::application::spawn_pipeline_task(&self.game_service, ctx, move |gs, ctx| {
            tracing::debug!("spawn_blocking: task started");
            let _guard = GenerationGuard(Arc::clone(&is_generating));
            if ctx.cancel_token.is_cancelled() {
                tracing::debug!("spawn_blocking: cancelled before execute_action");
                return;
            }
            execute_action_impl(gs, ctx, input);
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
