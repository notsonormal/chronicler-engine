use std::sync::Arc;

use crate::error::EngineError;
use crate::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, Confidence, ExecutionPhase, StatePatch,
};
use crate::narrative::agents::Agent;

use super::{QuantifierBackendTrait, determine_npcs_in_room, get_quantifier_backend};

pub struct QuantifierAgent {
    name: String,
    backend: Arc<dyn QuantifierBackendTrait>,
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
        let backend = Arc::from(get_quantifier_backend()) as Arc<dyn QuantifierBackendTrait>;
        Ok(Self {
            name: "quantifier".to_string(),
            backend,
        })
    }

    pub fn with_backend(name: String, backend: Arc<dyn QuantifierBackendTrait>) -> Self {
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
        let room_npc_ids = crate::engine::logic::get_current_room(state)
            .map(|r| r.npcs.clone())
            .unwrap_or_default();
        let previous_room_npcs: Vec<_> = state.scene.npcs_in_area.clone();

        let result = determine_npcs_in_room(
            state,
            &room_npc_ids,
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
