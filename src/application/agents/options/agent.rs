//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent implementation.

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, ExecutionPhase,
};

use crate::application::agents::Agent;
use crate::application::agents::options::utils::orchestration::generate_options;
use crate::application::llm_recorder::LlmCallRecorder;
#[cfg(feature = "testing")]
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::storage::Storage;

pub struct OptionsAgent {
    name: String,
    recorder: Arc<LlmCallRecorder>,
    storage: Option<Arc<Storage>>,
}

impl std::fmt::Debug for OptionsAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptionsAgent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl OptionsAgent {
    pub fn from_config_with_storage(
        _config: &AgentConfig,
        recorder: Arc<LlmCallRecorder>,
        storage: Option<Arc<Storage>>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "options".to_string(),
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

impl Agent for OptionsAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn phase(&self) -> ExecutionPhase {
        ExecutionPhase::OptionsGeneration
    }

    fn backend_selector(&self) -> BackendSelector {
        BackendSelector::UseNamed("options".to_string())
    }

    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError> {
        let options_prompt_override = {
            self.storage.as_ref().and_then(|s| {
                let settings = s.get_settings().ok()?;
                let preset_id = s.active_options_preset_id(&settings);
                s.get_preset(&preset_id)
                    .ok()
                    .flatten()
                    .map(|preset| preset.assemble_text(&[], None, None))
            })
        };

        let options = generate_options(ctx, self.recorder.as_ref(), options_prompt_override)?;

        Ok(AgentResult::Options(options))
    }
}
