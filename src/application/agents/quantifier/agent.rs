//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier agent implementation.

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, Confidence, ExecutionPhase, StatePatch,
};

use crate::application::agents::Agent;
use crate::application::llm_recorder::LlmCallRecorder;
#[cfg(feature = "testing")]
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::storage::Storage;

use super::determine_npcs_in_room;

pub struct QuantifierAgent {
    name: String,
    recorder: Arc<LlmCallRecorder>,
    storage: Option<Arc<Storage>>,
}

impl std::fmt::Debug for QuantifierAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuantifierAgent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl QuantifierAgent {
    pub fn from_config_with_storage(
        _config: &AgentConfig,
        recorder: Arc<LlmCallRecorder>,
        storage: Option<Arc<Storage>>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "quantifier".to_string(),
            recorder,
            storage,
        })
    }

    #[cfg(feature = "testing")]
    pub fn with_provider(name: String, provider: Arc<dyn LlmProvider>) -> Self {
        use crate::test_support::make_noop_save_fn;
        Self {
            name,
            recorder: Arc::new(LlmCallRecorder::new(provider, make_noop_save_fn())),
            storage: None,
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

        let quantifier_prompt_override = {
            self.storage.as_ref().and_then(|s| {
                let settings = s.get_settings().ok()?;
                let preset_id = s.active_quantifier_preset_id(&settings);
                s.get_preset(&preset_id)
                    .ok()
                    .flatten()
                    .map(|preset| preset.assemble_text(&[], None, None))
            })
        };

        let result = determine_npcs_in_room(
            ctx,
            main_response,
            self.recorder.as_ref(),
            quantifier_prompt_override,
        )?;

        let confidence = Confidence::from(result.npcs.confidence);

        Ok(AgentResult::StatePatch(StatePatch {
            npc_ids: result.npcs.npc_ids,
            movement_destination: result.movement.destination,
            confidence,
        }))
    }
}
