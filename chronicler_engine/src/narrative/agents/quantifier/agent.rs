use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, Confidence, ExecutionPhase, StatePatch,
};
use crate::model::settings::AppSettings;

use crate::narrative::agents::Agent;
use crate::storage::Storage;

use super::determine_npcs_in_room;

pub struct QuantifierAgent {
    name: String,
    backend: Arc<dyn crate::narrative::llm::LlmBackend>,
    preset_storage: Option<Arc<Storage>>,
    settings: Arc<RwLock<AppSettings>>,
}

impl std::fmt::Debug for QuantifierAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuantifierAgent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl QuantifierAgent {
    pub fn from_config(_config: &AgentConfig) -> Result<Self, EngineError> {
        Self::from_config_with_storage(
            _config,
            None,
            None,
            Arc::new(RwLock::new(AppSettings::default())),
        )
    }

    pub fn from_config_with_storage(
        _config: &AgentConfig,
        storage: Option<Arc<Storage>>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Result<Self, EngineError> {
        let settings_guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let backend = Arc::from(crate::narrative::llm::get_llm_backend_for(
            &settings_guard.quantifier_connection(),
            storage,
        ));
        Ok(Self {
            name: "quantifier".to_string(),
            backend,
            preset_storage,
            settings: Arc::clone(&settings),
        })
    }

    pub fn with_backend(name: String, backend: Arc<dyn crate::narrative::llm::LlmBackend>) -> Self {
        Self {
            name,
            backend,
            preset_storage: None,
            settings: Arc::new(RwLock::new(AppSettings::default())),
        }
    }
}

impl Agent for QuantifierAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn phase(&self) -> ExecutionPhase {
        ExecutionPhase::PostGeneration
    }

    fn backend_selector(&self) -> BackendSelector {
        BackendSelector::UseNamed("quantifier".to_string())
    }

    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError> {
        let main_response = ctx
            .main_response
            .ok_or_else(|| EngineError::Config("Quantifier requires main_response".into()))?;

        let state = ctx.state;
        let previous_room_npcs: Vec<_> = state.scene.npcs_in_area.clone();

        let current_room = ctx
            .current_room
            .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

        let quantifier_prompt_override = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            self.preset_storage
                .as_ref()
                .and_then(|s| {
                    s.get_preset(&settings.active_quantifier_prompt_preset_id)
                        .ok()
                        .flatten()
                })
                .map(|preset| preset.assemble_prompt_text(&[], None))
        };

        let result = determine_npcs_in_room(
            state,
            current_room,
            &[],
            &previous_room_npcs,
            main_response,
            self.backend.as_ref(),
            quantifier_prompt_override,
        );

        let confidence = Confidence::from(result.npcs.confidence);

        Ok(AgentResult::StatePatch(StatePatch::Scene {
            npc_ids: result.npcs.npc_ids,
            movement_destination: result.movement.destination,
            confidence,
        }))
    }
}
