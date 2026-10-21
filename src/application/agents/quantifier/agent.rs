//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier agent implementation.

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::agent::{AgentConfig, AgentSettings, HookInput};
use crate::domain::model::quantifier::QuantifierResult;

use crate::application::agents::trait_def::AfterNarrationAgent;
use crate::application::llm_recorder::LlmCallRecorder;
#[cfg(feature = "testing")]
use crate::application::ports::llm_provider::LlmProvider;

use super::types::QuantifierReading;
use super::utils::orchestration::determine_npcs_in_room;

pub struct QuantifierAgent {
    name: String,
    recorder: Arc<LlmCallRecorder>,
}

impl std::fmt::Debug for QuantifierAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuantifierAgent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl QuantifierAgent {
    pub fn from_config(
        _config: &AgentConfig,
        recorder: Arc<LlmCallRecorder>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "quantifier".to_string(),
            recorder,
        })
    }

    #[cfg(feature = "testing")]
    pub fn with_provider(name: String, provider: Arc<dyn LlmProvider>) -> Self {
        use crate::test_support::make_noop_save_fn;
        Self {
            name,
            recorder: Arc::new(LlmCallRecorder::new(provider, make_noop_save_fn())),
        }
    }
}

impl AfterNarrationAgent for QuantifierAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(
        &self,
        input: &HookInput,
        settings: &AgentSettings,
    ) -> Result<QuantifierResult, EngineError> {
        determine_npcs_in_room(input, self.recorder.as_ref(), settings)
            .map(QuantifierReading::into_result)
    }
}
