//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent implementation.

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::agent::{AgentConfig, AgentSettings, HookInput, OfferedSet};

use crate::application::agents::options::utils::orchestration::generate_options;
use crate::application::agents::trait_def::OptionsAgent;
use crate::application::llm_recorder::LlmCallRecorder;
#[cfg(feature = "testing")]
use crate::application::ports::llm_provider::LlmProvider;

pub struct OptionsGeneratorAgent {
    name: String,
    recorder: Arc<LlmCallRecorder>,
}

impl std::fmt::Debug for OptionsGeneratorAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptionsGeneratorAgent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl OptionsGeneratorAgent {
    pub fn from_config(
        _config: &AgentConfig,
        recorder: Arc<LlmCallRecorder>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "options".to_string(),
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

impl OptionsAgent for OptionsGeneratorAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<OfferedSet, EngineError> {
        let options = generate_options(input, self.recorder.as_ref(), settings)?;
        Ok(OfferedSet(options))
    }
}
