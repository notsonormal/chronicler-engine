//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! DefaultApplicationService — façade over application collaborators.

use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::adapters::driven::storage::Storage;
use crate::application::action_pipeline::phase_error::PhaseError;
use crate::application::action_pipeline::pipeline::ActionPipeline;
pub use crate::application::debug::DebugStateView;
pub use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::game_catalogue::GameCatalogue;
use crate::application::game_service::GameService;
use crate::application::game_view_query::GameViewQuery;
use crate::application::generation_gate::GenerationGate;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::spawn_pipeline_task;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::game::Game;
use crate::domain::model::map::MapDef;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

use crate::application::llm_message::LlmMessage;

#[derive(Clone)]
pub struct DefaultApplicationService {
    pub(crate) persistence_gate: Arc<PersistenceGate>,
    pub(crate) generation_gate: GenerationGate,
    pub(crate) game_catalogue: GameCatalogue,
    pub(crate) game_view_query: GameViewQuery,
    pub(crate) settings: Arc<RwLock<AppSettings>>,
    pub(crate) game_service: Arc<GameService>,
    pub(crate) pipeline: ActionPipeline,
    pub(crate) shutdown_token: CancellationToken,
}

impl DefaultApplicationService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        persistence_gate: Arc<PersistenceGate>,
        generation_gate: GenerationGate,
        game_catalogue: GameCatalogue,
        game_view_query: GameViewQuery,
        settings: Arc<RwLock<AppSettings>>,
        game_service: Arc<GameService>,
        pipeline: ActionPipeline,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            persistence_gate,
            generation_gate,
            game_catalogue,
            game_view_query,
            settings,
            game_service,
            pipeline,
            shutdown_token: cancel_token,
        }
    }

    pub fn game_service(&self) -> &Arc<GameService> {
        &self.game_service
    }

    /// Test seam only — no production callers.
    pub fn pipeline(&self) -> &ActionPipeline {
        &self.pipeline
    }

    pub fn storage(&self) -> &Arc<Storage> {
        self.persistence_gate.storage()
    }

    pub fn cancel_token(&self) -> &CancellationToken {
        &self.shutdown_token
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_token.is_cancelled()
    }

    pub fn settings(&self) -> &Arc<RwLock<AppSettings>> {
        &self.settings
    }

    pub fn switch_swipe(
        &self,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError> {
        let state = self.persistence_gate.load_or_fresh();
        let is_generating = state.narrative.input_buffer.status.is_generating();
        self.persistence_gate
            .switch_swipe(is_generating, message_id, swipe_index)
    }

    /// Test seam only — no production callers.
    pub fn active_quantifier_prompt(&self) -> String {
        self.game_view_query.active_quantifier_prompt()
    }

    pub fn process_action(
        self: &Arc<Self>,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = self.persistence_gate.load_or_fresh();
        let game_id = self.current_game_id();

        self.generation_gate.heal_stale(game_id, &mut game_state);

        let game = self.storage().require_game(game_id)?;
        let persona = self.storage().require_persona(&game.persona_key)?;
        let player_name = persona.sheet.name.clone();
        if !input.is_empty() {
            game_state.add_message(input.clone(), Some(player_name.clone()), MessageType::Input);
        }

        let (started_game_id, started_generation_id, claim_result) = self
            .generation_gate
            .try_claim(game_id, &mut game_state, &self.persistence_gate)?;
        match claim_result {
            ProcessActionResult::ConcurrentGeneration => {
                return Ok(ProcessActionResult::ConcurrentGeneration);
            }
            ProcessActionResult::Started => {}
            ProcessActionResult::ShuttingDown => {
                return Ok(ProcessActionResult::ShuttingDown);
            }
        }

        let gate = self.generation_gate.clone();
        spawn_pipeline_task(Arc::clone(self), move |app| {
            tracing::debug!("spawn_blocking: task started");
            let _guard = gate.guard(started_game_id, started_generation_id);
            let shutting = app.is_shutting_down();
            if shutting {
                tracing::debug!("spawn_blocking: shutting down before execute_action");
                return;
            }
            app.execute_action(input);
            tracing::debug!("spawn_blocking: execute_action completed");
        });
        Ok(ProcessActionResult::Started)
    }

    #[instrument(skip(self), fields(input_length))]
    pub fn execute_action(&self, input: String) {
        let mut state = self.persistence_gate.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.pipeline.run_from_input(state, input) {
            tracing::debug!("Pipeline cancelled");
        }
    }

    #[instrument(skip(self))]
    pub fn retry_last_response(&self) {
        let messages = match self.persistence_gate.load_messages() {
            Ok(m) => m,
            Err(e) => {
                self.pipeline
                    .retry_persist_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let Some((anchor_idx, _anchor_msg, snapshot_id)) =
            self.persistence_gate.find_retry_anchor(&messages)
        else {
            self.pipeline
                .retry_persist_error("Retry failed: no anchor message");
            return;
        };

        let is_event = messages
            .last()
            .map(|m| m.event_header().is_some())
            .unwrap_or(false);

        let old_target = messages
            .iter()
            .rev()
            .find(|m| {
                if is_event {
                    m.event_header().is_some()
                } else {
                    matches!(
                        m.message_type,
                        MessageType::Narration | MessageType::Dialogue
                    ) && m.event_header().is_none()
                }
            })
            .cloned();

        let snapshot = match self.storage().load_snapshot_by_id(snapshot_id) {
            Ok(Some(s)) => s,
            Ok(None) => {
                self.pipeline.retry_persist_error(format!(
                    "Retry failed: no snapshot found for id {snapshot_id}"
                ));
                return;
            }
            Err(e) => {
                self.pipeline
                    .retry_persist_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let mut state = GameState::from_snapshot(&snapshot);
        let mut truncated = messages;
        truncated.truncate(anchor_idx + 1);
        state.narrative.history.replace(truncated);
        state.narrative.retry_target = old_target;

        let input_text = match state.narrative.history.last_input_text() {
            Some((_, text)) => text,
            None => {
                self.pipeline
                    .retry_persist_error("Retry failed: no input to retry");
                return;
            }
        };

        let outcome = if is_event {
            self.pipeline.retry_event_continuation(state)
        } else {
            self.pipeline.retry_main_narration(state, input_text)
        };

        self.pipeline.handle_retry_outcome(outcome);
    }

    /// Test seam only — no production callers.
    #[instrument(skip(self))]
    pub fn retrigger_event(&self) {
        let state = self.persistence_gate.load_or_fresh();
        let outcome = self.pipeline.retry_event_continuation(state);
        self.pipeline.handle_retry_outcome(outcome);
    }

    pub fn continue_narration(self: &Arc<Self>) -> Result<ProcessActionResult, EngineError> {
        self.process_action(String::new())
    }

    pub fn retry(self: &Arc<Self>) -> Result<(), ApplicationError> {
        let game_state = self.persistence_gate.load_or_fresh();

        if game_state.narrative.history.last_input_text().is_none() {
            return Err(ApplicationError::validation("No input to retry"));
        }

        let (_, cancelled) = self.prepare_retry_state(
            game_state,
            GenerationStatus::Generating,
            GenerationPhase::Narrating,
        )?;
        if cancelled {
            return Err(ApplicationError::ShuttingDown);
        }

        spawn_pipeline_task(Arc::clone(self), move |app_inner| {
            if app_inner.is_shutting_down() {
                return;
            }
            app_inner.retry_last_response();
        });

        Ok(())
    }

    pub fn retrigger(self: &Arc<Self>) -> Result<(), ApplicationError> {
        let game_state = self.persistence_gate.load_or_fresh();

        if game_state.narrative.last_trigger.is_none() {
            return Err(ApplicationError::validation("No trigger context available"));
        }

        let messages = self.persistence_gate.load_messages()?;
        let Some(last_msg) = messages.last() else {
            return Err(ApplicationError::validation("No messages to retrigger"));
        };

        let is_narration = last_msg.message_type == MessageType::Narration
            || last_msg.message_type == MessageType::Dialogue;

        if !is_narration || last_msg.event_header().is_some() {
            return Err(ApplicationError::validation(
                "Last message must be a narration to retrigger",
            ));
        }

        let (_, cancelled) = self.prepare_retry_state(
            game_state,
            GenerationStatus::Generating,
            GenerationPhase::Narrating,
        )?;
        if cancelled {
            return Err(ApplicationError::ShuttingDown);
        }

        spawn_pipeline_task(Arc::clone(self), move |app_inner| {
            if app_inner.is_shutting_down() {
                return;
            }
            app_inner.retrigger_event();
        });

        Ok(())
    }

    pub fn create_game(&self, world_key: &str, persona_key: &str) -> Result<u64, ApplicationError> {
        self.game_catalogue.create_game(world_key, persona_key)
    }

    pub fn switch_game(&self, id: u64) -> Result<(), ApplicationError> {
        self.game_catalogue.switch_game(id)
    }

    pub fn delete_game(&self, id: u64) -> Result<(), ApplicationError> {
        self.game_catalogue.delete_game(id)
    }

    pub fn list_games(&self) -> Result<Vec<Game>, ApplicationError> {
        self.game_catalogue.list_games()
    }

    pub fn current_game_id(&self) -> u64 {
        self.game_catalogue.current_game_id()
    }

    pub fn reset(&self) -> Result<(), ApplicationError> {
        self.game_catalogue.reset()
    }

    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, ApplicationError> {
        self.persistence_gate
            .storage()
            .list_worlds()
            .map_err(Into::into)
    }

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, ApplicationError> {
        self.persistence_gate
            .storage()
            .get_world(key)
            .map_err(Into::into)
    }

    pub fn create_world(
        &self,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.persistence_gate
            .storage()
            .create_world(&world_card, &map)
            .map_err(Into::into)
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        self.persistence_gate
            .storage()
            .update_world(id, &world_card, &map)
            .map_err(Into::into)
    }

    pub fn delete_world(&self, key: &str) -> Result<(), ApplicationError> {
        self.persistence_gate
            .storage()
            .delete_world(key)
            .map_err(Into::into)
    }

    pub fn list_personas(&self) -> Result<Vec<PersonaCard>, ApplicationError> {
        self.persistence_gate
            .storage()
            .list_personas()
            .map_err(Into::into)
    }

    pub fn get_generating_status(
        &self,
    ) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
        self.game_view_query.get_generating_status()
    }

    pub fn reset_generating_status(&self) -> Result<(), ApplicationError> {
        self.generation_gate
            .reset_generating_status(&self.persistence_gate)
    }

    pub fn get_current_game_name(&self) -> Result<String, ApplicationError> {
        self.game_view_query.get_current_game_name()
    }

    pub fn list_latest_llm_messages(
        &self,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        self.game_view_query.list_latest_llm_messages(limit)
    }

    pub fn get_story_log_entries(&self) -> Result<(Vec<MessageEntry>, bool), ApplicationError> {
        self.game_view_query.get_story_log_entries()
    }

    pub fn get_current_room_view(&self) -> Result<(String, Option<String>), ApplicationError> {
        self.game_view_query.get_current_room_view()
    }

    pub fn get_npc_headshots(
        &self,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        self.game_view_query.get_npc_headshots(scene_only)
    }

    pub fn get_debug_state(&self) -> Result<DebugStateView, ApplicationError> {
        self.game_view_query.get_debug_state()
    }

    /// Set generation status/phase, persist the resulting snapshot, and return the
    /// mutated state paired with the current shutdown flag (`cancelled=true` if
    /// shutdown was requested mid-call).
    pub(crate) fn prepare_retry_state(
        &self,
        mut game_state: GameState,
        status: GenerationStatus,
        phase: GenerationPhase,
    ) -> Result<(GameState, bool), ApplicationError> {
        game_state.narrative.input_buffer.status = status;
        game_state.narrative.input_buffer.phase = phase;
        let snapshot = GameStateSnapshot::from_game_state(&game_state);
        self.storage().save_snapshot(&snapshot)?;
        let cancelled = self.is_shutting_down();
        Ok((game_state, cancelled))
    }
}
