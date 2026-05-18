use std::sync::{Arc, RwLock};

use crate::application::action_pipeline::ActionPipelineBackend;
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::model::quantifier::QuantifierConfidence;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::narrative::agents::quantifier::QuantifierAgent;
use crate::narrative::agents::registry::AgentRegistry;
use crate::narrative::llm::backend::{AGENT_NARRATOR, LlmCallResult};
use crate::storage::llm_message_storage::LlmMessageStorage;

pub trait GameService: Send + Sync {
    fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String);

    fn retry_last_response(&self, ctx: GameServiceContext);
}

pub struct DefaultGameService {
    pub(crate) llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
    pub(crate) agent_registry: AgentRegistry,
}

impl DefaultGameService {
    pub fn new() -> Self {
        Self::with_storage(None, Arc::new(RwLock::new(AppSettings::default())))
    }

    pub fn with_storage(
        storage: Option<Arc<dyn LlmMessageStorage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        let (registry, connection) = {
            let settings_guard = settings.read().unwrap_or_else(|e| e.into_inner());
            let registry = AgentRegistry::from_configs_with_storage(
                &settings_guard.agents,
                storage.clone(),
                &settings_guard,
            )
            .unwrap_or_default();
            (registry, settings_guard.narration_connection())
        };
        let llm_backend = Arc::from(crate::narrative::llm::get_llm_backend_for(
            &connection,
            storage,
            Some(Arc::clone(&settings)),
        ));
        Self {
            llm_backend,
            agent_registry: registry,
        }
    }

    pub fn with_backends(
        llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
        agent_registry: AgentRegistry,
    ) -> Self {
        Self {
            llm_backend,
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
    fn narrate_action(
        &self,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        self.llm_backend.narrate_action(AGENT_NARRATOR, context)
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

        for agent in self
            .agent_registry
            .agents_for_phase(ExecutionPhase::PostGeneration)
        {
            match agent.execute(&agent_ctx) {
                Ok(AgentResult::StatePatch(StatePatch::Scene {
                    npc_ids,
                    movement_destination,
                    confidence,
                })) => {
                    result.npcs.npc_ids = npc_ids;
                    result.movement.destination = movement_destination;
                    result.npcs.confidence = QuantifierConfidence::from(confidence);
                }
                Ok(AgentResult::NoOp) => {}
                Ok(AgentResult::PromptDirective(_)) => {
                    log::warn!("Post-generation agent returned PromptDirective; ignoring");
                }
                Err(e) => {
                    log::warn!("Agent {} failed: {e}", agent.name());
                }
            }
        }
    }
}

impl GameService for DefaultGameService {
    fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String) {
        crate::application::action_pipeline::execute_action_impl(self, ctx, input, player_name);
    }

    fn retry_last_response(&self, ctx: GameServiceContext) {
        crate::application::action_pipeline::retry_last_response_impl(self, ctx);
    }
}
