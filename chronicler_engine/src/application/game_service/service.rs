use std::sync::{Arc, RwLock};

use crate::application::action_pipeline::ActionPipelineBackend;
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::narrative::agents::quantifier::QuantifierAgent;
use crate::narrative::agents::registry::AgentRegistry;
use crate::narrative::llm::backend::LlmCallResult;
use crate::narrative::prompt::{LayeredPromptAssembler, PromptAssembler};
use crate::storage::Storage;

pub struct DefaultGameService {
    pub(crate) llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
    pub(crate) prompt_assembler: Arc<dyn PromptAssembler>,
    pub(crate) agent_registry: AgentRegistry,
}

impl DefaultGameService {
    pub fn new() -> Self {
        Self::with_storage(None, None, Arc::new(RwLock::new(AppSettings::default())))
    }

    pub fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String) {
        crate::application::action_pipeline::execute_action_impl(self, ctx, input, player_name);
    }

    pub fn retry_last_response(&self, ctx: GameServiceContext) {
        crate::application::action_pipeline::retry_last_response_impl(self, ctx);
    }

    pub fn retrigger_event(&self, ctx: GameServiceContext) {
        crate::application::action_pipeline::retrigger_event_impl(self, &ctx);
    }

    pub fn with_storage(
        storage: Option<Arc<Storage>>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        let (registry, connection, max_context_tokens, max_tokens) = {
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
            (registry, conn, max_context_tokens, max_tokens)
        };
        let llm_backend = Arc::from(crate::narrative::llm::get_llm_backend_for(
            &connection,
            storage,
        ));
        let mut assembler = LayeredPromptAssembler::new(max_context_tokens);
        if let Some(max) = max_tokens {
            assembler = assembler.with_max_tokens(max);
        }
        Self {
            llm_backend,
            prompt_assembler: Arc::new(assembler),
            agent_registry: registry,
        }
    }

    pub fn with_backends(
        llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
        agent_registry: AgentRegistry,
    ) -> Self {
        Self {
            llm_backend,
            prompt_assembler: Arc::new(LayeredPromptAssembler::new(
                crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS,
            )),
            agent_registry,
        }
    }

    pub fn with_mock_quantifier(
        llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
        quantifier_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
    ) -> Self {
        let agent = QuantifierAgent::with_backend("quantifier".to_string(), quantifier_backend);
        let registry = AgentRegistry::with_agent(Box::new(agent));
        Self {
            llm_backend,
            prompt_assembler: Arc::new(LayeredPromptAssembler::new(
                crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS,
            )),
            agent_registry: registry,
        }
    }
}

impl Default for DefaultGameService {
    fn default() -> Self {
        DefaultGameService::new()
    }
}

impl ActionPipelineBackend for DefaultGameService {
    fn assembler(&self) -> &dyn PromptAssembler {
        &*self.prompt_assembler
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        self.llm_backend
            .complete(agent_name, system_prompt, user_prompt, max_tokens)
    }

    fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
        result: &mut crate::model::quantifier::QuantifierResult,
    ) {
        let agent_ctx = AgentContext {
            state,
            main_response: Some(main_response),
            player_input,
            current_room: state.current_room(),
        };

        let patch = self
            .agent_registry
            .agents_for_phase(ExecutionPhase::PostGeneration)
            .filter_map(|agent| match agent.execute(&agent_ctx) {
                Ok(AgentResult::StatePatch(patch)) => Some(patch),
                Ok(AgentResult::NoOp) | Ok(AgentResult::PromptDirective(_)) => None,
                Err(e) => {
                    log::warn!("Agent {} failed: {e}", agent.name());
                    None
                }
            })
            .fold(
                StatePatch::Scene {
                    npc_ids: result.npcs.npc_ids.clone(),
                    movement_destination: result.movement.destination.clone(),
                    confidence: result.npcs.confidence.clone().into(),
                },
                StatePatch::merge,
            );

        let StatePatch::Scene {
            npc_ids,
            movement_destination,
            confidence,
        } = patch;
        result.npcs.npc_ids = npc_ids;
        result.movement.destination = movement_destination;
        result.npcs.confidence = confidence.into();
    }
}
