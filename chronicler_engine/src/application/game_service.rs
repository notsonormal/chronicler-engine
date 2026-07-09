//! [DOC: docs/system/game_flow.md]
//! Game service handling gameplay operations
//! arch-lint: storage-direct — intentional, see ADR-027

use std::sync::{Arc, RwLock};

use crate::domain::model::settings::AppSettings;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::narrative_prompt::PromptAssembler;
use crate::application::llm_recorder::LlmCallRecorder;

#[derive(Clone)]
pub struct GameService {
    pub llm_recorder: Arc<LlmCallRecorder>,
    pub prompt_assembler: Arc<PromptAssembler>,
    pub agent_registry: Arc<AgentRegistry>,
}

impl GameService {
    pub fn with_storage(
        llm_recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        let (max_context_tokens, max_tokens) = {
            let guard = settings.read().unwrap_or_else(|e| e.into_inner());
            let conn = guard.narration_connection();
            (conn.resolve_max_context_tokens(), conn.max_tokens)
        };
        let mut assembler = PromptAssembler::new(max_context_tokens);
        if let Some(max) = max_tokens {
            assembler = assembler.with_max_tokens(max);
        }
        tracing::info!(
            "GameService: backend={}, model={}",
            llm_recorder.provider().name(),
            llm_recorder.provider().model()
        );
        Self {
            llm_recorder,
            prompt_assembler: Arc::new(assembler),
            agent_registry: Arc::new(agent_registry),
        }
    }

    pub fn with_backends(
        llm_recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
    ) -> Self {
        Self {
            llm_recorder,
            prompt_assembler: Arc::new(PromptAssembler::new(
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
            prompt_assembler: Arc::new(PromptAssembler::new(
                crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
            )),
            agent_registry: Arc::new(registry),
        }
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
