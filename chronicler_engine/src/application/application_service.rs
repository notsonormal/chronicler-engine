//! [DOC: docs/system/game_flow.md]
//! DefaultApplicationService — thin façade over 4 cohesive modules plus 2 collaborator fields (T2 ticket 04 — final façade shrink).

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::storage::worlds::WorldWithMap;
use crate::adapters::driven::storage::{PresetStore, Storage};
pub use crate::application::debug::DebugStateView;
pub use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::game_catalogue::GameCatalogue;
use crate::application::game_service::GameService;
use crate::application::generation_gate::GenerationGate;
pub use crate::application::mappers::map_llm_error;
use crate::application::persistence_gate::{PersistenceGate, WorldSnapshot};
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::game::Game;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

use crate::application::narrative_prompt::assembler::assemble_prompt_text;

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

#[derive(Clone)]
#[allow(dead_code)]
pub struct DefaultApplicationService {
    pub(crate) persistence_gate: Arc<PersistenceGate>,
    pub(crate) generation_gate: GenerationGate,
    pub(crate) game_catalogue: GameCatalogue,
    pub(crate) world_catalogue: WorldCatalogue,
    pub(crate) settings: Arc<RwLock<AppSettings>>,
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
        let preset_store = Arc::new(PresetStore::new(preset_storage));
        let persistence_gate = Arc::new(PersistenceGate::new(
            Arc::clone(&storage),
            Arc::clone(&preset_store),
        ));
        let generation_gate = GenerationGate::new(cancel_token, Arc::clone(&is_generating));
        // is_generating: Arc<AtomicBool> direct per ADR-030 hot-path; not behind GenerationGate::is_generating() accessor.
        let game_catalogue = GameCatalogue::new(Arc::clone(&persistence_gate), is_generating);
        // raw storage: WorldCatalogue owns worlds persistence directly; deliberate asymmetry vs GameCatalogue (which borrows Arc<PersistenceGate>) to keep the game/world seams independent.
        let world_catalogue = WorldCatalogue::new(storage);
        Self {
            persistence_gate,
            generation_gate,
            game_catalogue,
            world_catalogue,
            settings,
            game_service,
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
        self.generation_gate.cancel_token()
    }

    pub fn settings(&self) -> &Arc<RwLock<AppSettings>> {
        &self.settings
    }

    pub fn preset_storage(&self) -> &Arc<PresetStore> {
        self.persistence_gate.preset_store()
    }

    pub(crate) fn load_world_snapshot(&self) -> Result<WorldSnapshot, EngineError> {
        self.persistence_gate.load_world_snapshot()
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

    pub fn active_quantifier_prompt(&self) -> String {
        let preset_id = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            settings.active_quantifier_prompt_preset_id.clone()
        };
        match self.preset_storage().get_preset(&preset_id) {
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
}
