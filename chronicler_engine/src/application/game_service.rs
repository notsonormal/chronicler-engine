//! [DOC: docs/system/game_flow.md]
//! Game service handling gameplay operations
//! arch-lint: storage-direct — intentional, see ADR-027

use std::sync::{Arc, RwLock};

use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::domain::model::settings::AppSettings;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::narrative_prompt::LayeredPromptAssembler;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::storage::Storage;

pub struct GameService {
    pub llm_recorder: Arc<LlmCallRecorder>,
    pub prompt_assembler: Arc<LayeredPromptAssembler>,
    pub agent_registry: Arc<AgentRegistry>,
}

impl GameService {
    pub fn with_storage(
        storage: Option<Arc<Storage>>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Result<Self, EngineError> {
        let (registry, connection, max_context_tokens, max_tokens, storage) = {
            let settings_guard = settings.read().unwrap_or_else(|e| e.into_inner());
            let registry = AgentRegistry::from_configs_with_storage(
                &settings_guard.agents,
                storage.clone(),
                preset_storage,
                Arc::clone(&settings),
            )
            .unwrap_or_default();
            let conn = settings_guard.narration_connection();
            let max_context_tokens = conn.resolve_max_context_tokens();
            let max_tokens = conn.max_tokens;
            // Use provided storage or fresh storage
            let storage = storage.unwrap_or_else(|| Arc::new(Storage::new_in_memory()));
            (registry, conn, max_context_tokens, max_tokens, storage)
        };
        let llm_recorder =
            crate::bootstrap::llm_factory::get_llm_recorder_for(&connection, Arc::clone(&storage))?;
        tracing::info!(
            "GameService: backend={}, model={}",
            llm_recorder.provider().name(),
            llm_recorder.provider().model()
        );
        let mut assembler = LayeredPromptAssembler::new(max_context_tokens);
        if let Some(max) = max_tokens {
            assembler = assembler.with_max_tokens(max);
        }
        Ok(Self {
            llm_recorder,
            prompt_assembler: Arc::new(assembler),
            agent_registry: Arc::new(registry),
        })
    }

    pub fn with_backends(
        llm_recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
    ) -> Self {
        Self {
            llm_recorder,
            prompt_assembler: Arc::new(LayeredPromptAssembler::new(
                crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
            )),
            agent_registry: Arc::new(agent_registry),
        }
    }

    pub fn with_mock_quantifier(
        llm_recorder: Arc<LlmCallRecorder>,
        quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    ) -> Self {
        let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
        let registry = AgentRegistry::with_agent(Box::new(agent));
        Self {
            llm_recorder,
            prompt_assembler: Arc::new(LayeredPromptAssembler::new(
                crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
            )),
            agent_registry: Arc::new(registry),
        }
    }

    pub fn execute_action(&self, ctx: GameServiceContext, input: String) {
        crate::application::action_pipeline::execute_action_impl(self, ctx, input)
    }

    pub fn retry_last_response(&self, ctx: GameServiceContext) {
        crate::application::action_pipeline::retry_last_response_impl(self, ctx)
    }

    pub fn backend_info(&self) -> (&str, &str) {
        (
            self.llm_recorder.provider().name(),
            self.llm_recorder.provider().model(),
        )
    }

    pub fn pipeline(&self) -> crate::application::action_pipeline::pipeline::ActionPipeline {
        crate::application::action_pipeline::pipeline::ActionPipeline::new(
            Arc::clone(&self.prompt_assembler),
            Arc::clone(&self.llm_recorder),
            Arc::clone(&self.agent_registry),
        )
    }
}
