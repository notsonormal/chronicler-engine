//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent implementation.

use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::domain::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, ExecutionPhase,
};
use crate::domain::model::settings::AppSettings;

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
    settings: Arc<RwLock<AppSettings>>,
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
        settings: Arc<RwLock<AppSettings>>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "options".to_string(),
            recorder,
            storage,
            settings: Arc::clone(&settings),
        })
    }

    #[cfg(feature = "testing")]
    pub fn with_provider(name: String, provider: Arc<dyn LlmProvider>) -> Self {
        use crate::test_support::make_noop_save_fn;
        Self {
            name,
            recorder: Arc::new(LlmCallRecorder::new(provider, make_noop_save_fn())),
            storage: None,
            settings: Arc::new(RwLock::new(AppSettings::default())),
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
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            self.storage.as_ref().and_then(|s| {
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
