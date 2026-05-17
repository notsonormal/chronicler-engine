use std::sync::Arc;

use crate::error::EngineError;
use crate::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, Confidence, ExecutionPhase, StatePatch,
};

use crate::narrative::agents::Agent;

use super::determine_npcs_in_room;

pub struct QuantifierAgent {
    name: String,
    backend: Arc<dyn crate::narrative::llm::LlmBackend>,
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
            &crate::model::settings::AppSettings::default(),
        )
    }

    pub fn from_config_with_storage(
        _config: &AgentConfig,
        storage: Option<Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage>>,
        settings: &crate::model::settings::AppSettings,
    ) -> Result<Self, EngineError> {
        let backend = Arc::from(crate::narrative::llm::get_llm_backend_for(
            &settings.quantifier_connection(),
            storage,
            None,
        ));
        Ok(Self {
            name: "quantifier".to_string(),
            backend,
        })
    }

    pub fn with_backend(name: String, backend: Arc<dyn crate::narrative::llm::LlmBackend>) -> Self {
        Self { name, backend }
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

        let result = determine_npcs_in_room(
            state,
            &[],
            &previous_room_npcs,
            main_response,
            self.backend.as_ref(),
        );

        let confidence = Confidence::from(result.npcs.confidence);

        Ok(AgentResult::StatePatch(StatePatch::Scene {
            npc_ids: result.npcs.npc_ids,
            movement_destination: result.movement.destination,
            confidence,
        }))
    }
}
