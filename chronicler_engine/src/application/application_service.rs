//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! DefaultApplicationService — façade over application collaborators.

use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::adapters::driven::storage::{PresetStore, Storage};
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
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::game::Game;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

use crate::application::llm_message::LlmMessage;

#[derive(Clone)]
#[allow(dead_code)]
pub struct DefaultApplicationService {
    pub(crate) persistence_gate: Arc<PersistenceGate>,
    pub(crate) generation_gate: GenerationGate,
    pub(crate) game_catalogue: GameCatalogue,
    pub(crate) game_view_query: GameViewQuery,
    pub(crate) world_catalogue: WorldCatalogue,
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
        world_catalogue: WorldCatalogue,
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
            world_catalogue,
            settings,
            game_service,
            pipeline,
            shutdown_token: cancel_token,
        }
    }

    pub fn game_service(&self) -> &Arc<GameService> {
        &self.game_service
    }

    pub fn pipeline(&self) -> &ActionPipeline {
        &self.pipeline
    }

    pub fn storage(&self) -> &Arc<Storage> {
        self.persistence_gate.storage()
    }

    /// Cached projection of `GenerationStatus::Generating`. Read-only on the facade;
    /// the underlying `Arc<AtomicBool>` stays internal to `GenerationGate`
    /// (ADR-030) so callers can't mutate generation flag state through the API boundary.
    pub fn is_generating_now(&self) -> bool {
        self.generation_gate.is_generating().load(Ordering::SeqCst)
    }

    /// Test seam: force the cached projection flag. Production reads should go
    /// through `is_generating_now()`; this exists so tests can simulate an
    /// in-flight generation without exposing `GenerationGate`'s internal atomic.
    pub fn set_is_generating(&self, value: bool) {
        self.generation_gate
            .is_generating()
            .store(value, Ordering::SeqCst);
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

    pub fn preset_storage(&self) -> &Arc<PresetStore> {
        self.persistence_gate.preset_store()
    }

    pub fn load_or_fresh(&self) -> GameState {
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

    pub fn switch_swipe(
        &self,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError> {
        self.persistence_gate
            .switch_swipe(self.is_generating_now(), message_id, swipe_index)
    }

    pub fn edit_history(&self, id: u64, text: String) -> Result<(), ApplicationError> {
        self.persistence_gate.edit_history(id, text)
    }

    pub fn delete_last(&self) -> Result<(), ApplicationError> {
        self.persistence_gate.delete_last()
    }

    pub fn active_quantifier_prompt(&self) -> String {
        self.game_view_query.active_quantifier_prompt()
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

    pub fn process_action(
        self: &Arc<Self>,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = self.load_or_fresh();
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
        let mut state = self.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.pipeline.run_from_input(state, input) {
            tracing::debug!("Pipeline cancelled");
        }
    }

    #[instrument(skip(self))]
    pub fn retry_last_response(&self) {
        let messages = match self.load_messages() {
            Ok(m) => m,
            Err(e) => {
                self.pipeline
                    .retry_persist_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let Some((anchor_idx, _anchor_msg, snapshot_id)) = self.find_retry_anchor(&messages) else {
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

    #[instrument(skip(self))]
    pub fn retrigger_event(&self) {
        let state = self.load_or_fresh();
        let outcome = self.pipeline.retry_event_continuation(state);
        self.pipeline.handle_retry_outcome(outcome);
    }

    pub fn continue_narration(self: &Arc<Self>) -> Result<ProcessActionResult, EngineError> {
        self.process_action(String::new())
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
        self.world_catalogue.list_worlds()
    }

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, ApplicationError> {
        self.world_catalogue.get_world(key)
    }

    pub fn create_world(
        &self,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<i64, ApplicationError> {
        self.world_catalogue.create_world(world_card, map)
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: WorldCard,
        map: MapDef,
    ) -> Result<(), ApplicationError> {
        self.world_catalogue.update_world(id, world_card, map)
    }

    pub fn delete_world(&self, key: &str) -> Result<(), ApplicationError> {
        self.world_catalogue.delete_world(key)
    }

    pub fn list_personas(&self) -> Result<Vec<PersonaCard>, ApplicationError> {
        self.world_catalogue.list_personas()
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

    /// Returns `cancelled=true` if shutdown was requested mid-call.
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
