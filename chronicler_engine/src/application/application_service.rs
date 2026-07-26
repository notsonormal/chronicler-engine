//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! DefaultApplicationService — façade over application collaborators.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::adapters::driven::storage::{PresetStore, Storage};
use crate::application::action_pipeline::phase_error::PhaseError;
pub use crate::application::debug::DebugStateView;
pub use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::game_catalogue::GameCatalogue;
use crate::application::game_service::GameService;
use crate::application::game_view_query::GameViewQuery;
use crate::application::generation_gate::GenerationGate;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::game::Game;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
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
    pub(crate) shutdown_token: CancellationToken,
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
        let preset_store = Arc::new(PresetStore::new(preset_storage));
        let persistence_gate = Arc::new(PersistenceGate::new(
            Arc::clone(&storage),
            Arc::clone(&preset_store),
        ));
        let generation_gate = GenerationGate::new(Arc::clone(&is_generating));
        // Direct atomic access per ADR-030 hot-path.
        let game_catalogue = GameCatalogue::new(Arc::clone(&persistence_gate));
        let game_view_query =
            GameViewQuery::new(Arc::clone(&persistence_gate), Arc::clone(&settings));
        // WorldCatalogue owns worlds persistence directly (asymmetric vs GameCatalogue) to keep seams independent.
        let world_catalogue = WorldCatalogue::new(storage);
        Self {
            persistence_gate,
            generation_gate,
            game_catalogue,
            game_view_query,
            world_catalogue,
            settings,
            game_service,
            shutdown_token: cancel_token,
        }
    }

    pub fn game_service(&self) -> &Arc<GameService> {
        &self.game_service
    }

    pub fn storage(&self) -> &Arc<Storage> {
        self.persistence_gate.storage()
    }

    pub fn is_generating(&self) -> &Arc<AtomicBool> {
        self.generation_gate.is_generating()
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
            .switch_swipe(self.is_generating(), message_id, swipe_index)
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

    pub fn process_action(&self, input: String) -> Result<ProcessActionResult, EngineError> {
        self.generation_gate.start_action(self, input)
    }

    #[instrument(skip(self), fields(input_length))]
    pub fn execute_action(&self, input: String) {
        let mut state = self.load_or_fresh();
        state.narrative.last_trigger = None;
        let pipeline = self.game_service().pipeline();
        if let Err(PhaseError::Cancelled) = pipeline.run_from_input(self, state, input) {
            tracing::debug!("Pipeline cancelled");
        }
    }

    #[instrument(skip(self))]
    pub fn retry_last_response(&self) {
        let messages = match self.load_messages() {
            Ok(m) => m,
            Err(e) => {
                self.retry_persist_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let Some((anchor_idx, _anchor_msg, snapshot_id)) = self.find_retry_anchor(&messages) else {
            self.retry_persist_error("Retry failed: no anchor message");
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
                self.retry_persist_error(format!(
                    "Retry failed: no snapshot found for id {snapshot_id}"
                ));
                return;
            }
            Err(e) => {
                self.retry_persist_error(format!("Retry failed: {e}"));
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
                self.retry_persist_error("Retry failed: no input to retry");
                return;
            }
        };

        let outcome = if is_event {
            self.retry_event_continuation(state)
        } else {
            self.retry_main_narration(state, input_text)
        };

        self.handle_retry_outcome(outcome);
    }

    #[instrument(skip(self))]
    pub fn retrigger_event(&self) {
        let state = self.load_or_fresh();
        let outcome = self.retry_event_continuation(state);
        self.handle_retry_outcome(outcome);
    }

    #[allow(dead_code)]
    pub(crate) fn heal_stale_generating(&self, state: &mut GameState) {
        self.generation_gate.heal_stale_generating(self, state)
    }

    pub fn continue_narration(&self) -> Result<ProcessActionResult, EngineError> {
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
}
