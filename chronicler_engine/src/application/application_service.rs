//! [DOC: docs/system/game_flow.md]
//! Main application service coordinating game operations
//! arch-lint: storage-direct — intentional, see ADR-027

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::application::action_pipeline::execute_action_impl;
use crate::application::context::{OpContext, load_or_fresh};
use crate::application::game_service::GameService;
use crate::application::generation_guard::GenerationGuard;

use crate::error::EngineError;
use crate::domain::model::game::{Game, generate_game_name};
use crate::domain::model::settings::AppSettings;

use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::trigger::NpcEncounterState;
use crate::domain::model::world::WorldCard;
use crate::domain::model::map::MapDef;
use crate::adapters::driven::storage::Storage;
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

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn load_state_for_test(&self) -> GameState {
        let ctx = self.synthesize_ctx();
        crate::application::context::load_state_for_test(&ctx)
    }

    pub(crate) fn load_world_snapshot(&self) -> Result<crate::application::context::WorldSnapshot, EngineError> {
        let game_id = self.storage.current_game_id();
        let game = self.storage.get_game(game_id)?
            .ok_or_else(|| EngineError::Config(format!("current_game_id {} not found", game_id)))?;
        let world_with_map = self.storage.get_world(&game.world_key)?
            .ok_or_else(|| EngineError::Config(format!("world '{}' not found", game.world_key)))?;
        let player = self.storage.get_persona(&game.persona_key)?
            .ok_or_else(|| EngineError::Config(format!("persona '{}' not found", game.persona_key)))?;
        let npcs_list = self.storage.list_characters(world_with_map.world_id)?;
        let mut npcs: std::collections::HashMap<String, crate::domain::model::character::NpcCard> = std::collections::HashMap::new();
        for n in npcs_list {
            npcs.insert(n.id.clone(), n);
        }
        Ok(crate::application::context::WorldSnapshot {
            world: Arc::new(world_with_map.world_card),
            map: Arc::new(world_with_map.map),
            player: Arc::new(player),
            npcs: Arc::new(npcs),
        })
    }

    pub(crate) fn synthesize_ctx(&self) -> OpContext {
        let snapshot = self.load_world_snapshot().unwrap_or_else(|e| {
            panic!("synthesize_ctx: failed to load world snapshot: {e}")
        });
        OpContext {
            storage: self.storage.clone(),
            preset_storage: self.preset_storage.clone(),
            settings: self.settings.clone(),
            cancel_token: self.cancel_token.clone(),
            is_generating: self.is_generating.clone(),
            world_snapshot: snapshot,
        }
    }

    pub fn load_or_fresh(&self) -> Result<GameState, EngineError> {
        let snapshot = self.load_world_snapshot();
        let ctx = match snapshot {
            Ok(s) => OpContext {
                storage: self.storage.clone(),
                preset_storage: self.preset_storage.clone(),
                settings: self.settings.clone(),
                cancel_token: self.cancel_token.clone(),
                is_generating: self.is_generating.clone(),
                world_snapshot: s,
            },
            Err(e) => return Err(e),
        };
        Ok(crate::application::context::load_or_fresh(&ctx))
    }
    pub fn load_expecting_valid_state(&self) -> Result<GameState, EngineError> {
        let ctx = self.synthesize_ctx();
        crate::application::context::load_expecting_valid_state(&ctx)
    }

    pub fn save_state(&self, state: &GameState) -> Result<u64, EngineError> {
        let ctx = self.synthesize_ctx();
        crate::application::context::save_state(&ctx, state)
    }

    pub fn save_message_and_snapshot(&self, state: &mut GameState) -> Result<u64, EngineError> {
        let ctx = self.synthesize_ctx();
        crate::application::context::save_message_and_snapshot(&ctx, state)
    }

    pub fn delete_and_remove_message(&self, state: &mut GameState, id: u64) -> Result<(), EngineError> {
        let ctx = self.synthesize_ctx();
        crate::application::context::delete_and_remove_message(&ctx, state, id)
    }

    pub fn load_messages_with_swipes(&self) -> Result<Vec<crate::domain::model::message::Message>, EngineError> {
        crate::application::context::load_messages_with_swipes(&self.storage)
    }

    pub fn load_messages_into_state(&self, state: &mut GameState) {
        if let Ok(msgs) = self.load_messages() {
            state.narrative.history.replace(msgs);
        }
    }

    pub fn build_fresh_initial_state(&self) -> Result<GameState, EngineError> {
        let ctx = self.synthesize_ctx();
        Ok(ctx.build_fresh_initial_state())
    }

    pub fn load_messages(&self) -> Result<Vec<crate::domain::model::message::Message>, EngineError> {
        crate::application::context::load_messages_with_swipes(&self.storage)
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
            Ok(Some(preset)) => crate::application::narrative_prompt::assembler::assemble_prompt_text(&preset, &[], None),
            Ok(None) => {
                tracing::error!("active quantifier preset '{preset_id}' not found — defaults not seeded?");
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
        messages: &'a [crate::domain::model::message::Message],
    ) -> Option<(usize, &'a crate::domain::model::message::Message, u64)> {
        let ctx = self.synthesize_ctx();
        ctx.find_retry_anchor(messages)
    }

    pub fn set_game_id(&self, game_id: u64) {
        self.storage.set_game_id(game_id);
    }

    pub fn process_action(
        &self,
        ctx: OpContext,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = load_or_fresh(&ctx);

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

    pub fn continue_narration(&self, ctx: OpContext) -> Result<ProcessActionResult, EngineError> {
        self.process_action(ctx, String::new())
    }

    pub fn create_game(&self, ctx: OpContext) -> Result<u64, ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let world_with_map = ctx
            .storage
            .get_world(&ctx.world_snapshot.world.key)?
            .ok_or_else(|| ApplicationError::validation("World not found"))?;
        let world_name = world_with_map.world_card.name.clone();
        let games = ctx.storage.list_games()?;
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);

        let new_id = ctx.storage.create_game(
            &world_name,
            &ctx.world_snapshot.world.key,
            &ctx.world_snapshot.player.key,
            &ctx.world_snapshot.player.sheet.name,
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

    pub fn switch_game(&self, ctx: OpContext, id: u64) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        if ctx.storage.get_game(id)?.is_none() {
            return Err(ApplicationError::validation("Game not found"));
        }

        ctx.set_game_id(id);
        Ok(())
    }

    pub fn delete_game(&self, ctx: OpContext, id: u64) -> Result<(), ApplicationError> {
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

    pub fn list_games(&self, ctx: OpContext) -> Result<Vec<Game>, ApplicationError> {
        ctx.storage.list_games().map_err(Into::into)
    }

    pub fn current_game_id(&self, ctx: OpContext) -> u64 {
        ctx.storage.current_game_id()
    }

    pub fn reset(&self, ctx: OpContext) -> Result<(), ApplicationError> {
        let current_id = ctx.storage.current_game_id();
        let world_key = ctx.world_snapshot.world.key.clone();
        let world_name = ctx.world_snapshot.world.name.clone();

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
            &ctx.world_snapshot.player.key,
            &ctx.world_snapshot.player.sheet.name,
            &new_name,
        )?;
        ctx.set_game_id(new_id);

        let _ = Self::persist_initial_state_with_swipes(&ctx);

        Ok(())
    }

    pub fn list_worlds(&self, ctx: OpContext) -> Result<Vec<WorldCard>, ApplicationError> {
        ctx.storage.list_worlds().map_err(Into::into)
    }

    pub fn get_world(
        &self,
        ctx: OpContext,
        key: &str,
    ) -> Result<Option<WorldWithMap>, ApplicationError> {
        ctx.storage.get_world(key).map_err(Into::into)
    }

    pub fn create_world(
        &self,
        ctx: OpContext,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        ctx.storage
            .create_world(&world_card, &map)
            .map_err(Into::into)
    }

    pub fn update_world(
        &self,
        ctx: OpContext,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        ctx.storage
            .update_world(id, &world_card, &map)
            .map_err(Into::into)
    }

    pub fn delete_world(&self, ctx: OpContext, key: &str) -> Result<(), ApplicationError> {
        ctx.storage.delete_world(key).map_err(Into::into)
    }

    fn persist_initial_state_with_swipes(ctx: &OpContext) -> Result<u64, ApplicationError> {
        let mut initial_state = ctx.build_fresh_initial_state();
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
