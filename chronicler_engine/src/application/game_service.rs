//! [DOC: docs/system/game_flow.md]
//! Game service handling gameplay operations

use std::sync::{Arc, RwLock};

use crate::application::action_pipeline::ActionPipelineBackend;
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::domain::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::ports::llm_provider::LlmCallResult;
use crate::application::narrative_prompt::LayeredPromptAssembler;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::storage::Storage;

pub struct GameService {
    pub(crate) llm_recorder: Arc<LlmCallRecorder>,
    pub(crate) prompt_assembler: Arc<LayeredPromptAssembler>,
    pub(crate) agent_registry: AgentRegistry,
}

impl GameService {
    pub fn new() -> Self {
        Self::with_storage(None, None, Arc::new(RwLock::new(AppSettings::default())))
    }

    pub fn with_storage(
        storage: Option<Arc<Storage>>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
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
        let llm_recorder = crate::bootstrap::llm_factory::get_llm_recorder_for(&connection, Arc::clone(&storage))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to create LLM recorder: {e}");
                // Fallback to mock
                struct NoopForensics;
                impl crate::application::ports::llm_message_repository::LlmMessageRepository for NoopForensics {
                    fn save_llm_message(&self, _message: &crate::application::ports::llm_message_repository::LlmMessage) -> Result<(), EngineError> { Ok(()) }
                    fn list_latest_llm_messages(&self, _limit: usize) -> Result<Vec<crate::application::ports::llm_message_repository::LlmMessage>, EngineError> { Ok(Vec::new()) }
                }
                Arc::new(crate::application::llm_recorder::LlmCallRecorder::new(
                    Arc::new(crate::adapters::driven::llm::providers::MockBackend::new(Some(Arc::clone(&storage)))),
                    Arc::new(NoopForensics),
                ))
            });
        tracing::info!(
            "GameService: backend={}, model={}",
            llm_recorder.provider().name(),
            llm_recorder.provider().model()
        );
        let mut assembler = LayeredPromptAssembler::new(max_context_tokens);
        if let Some(max) = max_tokens {
            assembler = assembler.with_max_tokens(max);
        }
        Self {
            llm_recorder,
            prompt_assembler: Arc::new(assembler),
            agent_registry: registry,
        }
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
            agent_registry,
        }
    }

    pub fn with_mock_quantifier(
        llm_recorder: Arc<LlmCallRecorder>,
        quantifier_recorder: Arc<LlmCallRecorder>,
    ) -> Self {
        let agent = QuantifierAgent::with_backend(
            "quantifier".to_string(),
            quantifier_recorder.provider().clone(),
        );
        let registry = AgentRegistry::with_agent(Box::new(agent));
        Self {
            llm_recorder,
            prompt_assembler: Arc::new(LayeredPromptAssembler::new(
                crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
            )),
            agent_registry: registry,
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
}

impl Default for GameService {
    fn default() -> Self {
        GameService::new()
    }
}

impl ActionPipelineBackend for GameService {
    fn assembler(&self) -> &LayeredPromptAssembler {
        &self.prompt_assembler
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        self.llm_recorder
            .complete(agent_name, system_prompt, user_prompt, max_tokens)
    }

    fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
        result: &mut crate::domain::model::quantifier::QuantifierResult,
    ) {
        let agent_ctx = AgentContext {
            state,
            main_response: Some(main_response),
            player_input,
            current_room: state.current_room(),
        };

        let patches: Vec<_> = self
            .agent_registry
            .agents_for_phase(ExecutionPhase::PostGeneration)
            .filter_map(|agent| match agent.execute(&agent_ctx) {
                Ok(AgentResult::StatePatch(patch)) => Some(patch),
                Ok(AgentResult::NoOp) | Ok(AgentResult::PromptDirective(_)) => None,
                Err(e) => {
                    tracing::warn!("Agent {} failed: {e}", agent.name());
                    None
                }
            })
            .collect();

        if let Some(first_patch) = patches.into_iter().reduce(StatePatch::merge) {
            let StatePatch {
                npc_ids,
                movement_destination,
                confidence,
            } = first_patch;
            result.npcs.npc_ids = npc_ids;
            result.movement.destination = movement_destination;
            result.npcs.confidence = confidence.into();
        }
    }
}
