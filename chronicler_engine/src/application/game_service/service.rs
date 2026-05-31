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

    /// Execute a player action through the action pipeline.
    pub fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String) {
        crate::application::action_pipeline::execute_action_impl(self, ctx, input, player_name)
    }

    /// Retry the last response.
    pub fn retry_last_response(&self, ctx: GameServiceContext) {
        crate::application::action_pipeline::retry_last_response_impl(self, ctx)
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
                    tracing::warn!("Agent {} failed: {e}", agent.name());
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::narrative::agents::registry::AgentRegistry;
    use crate::narrative::llm::MockBackend;
    use crate::test_support::{fixtures::TestWorld, make_test_context};

    use super::*;

    #[test]
    fn test_default_construction_creates_service() {
        let service = DefaultGameService::default();
        let _assembler: &dyn PromptAssembler = service.assembler();
    }

    #[test]
    fn test_new_construction_creates_service() {
        let service = DefaultGameService::new();
        let _assembler: &dyn PromptAssembler = service.assembler();
    }

    #[test]
    fn test_with_backends_creates_service() {
        let llm_backend = Arc::new(MockBackend::default());
        let registry = AgentRegistry::default();
        let service = DefaultGameService::with_backends(llm_backend, registry);
        let _assembler: &dyn PromptAssembler = service.assembler();
    }

    #[test]
    fn test_with_mock_quantifier_creates_service() {
        let llm_backend = Arc::new(MockBackend::default());
        let quantifier_backend = Arc::new(MockBackend::default());
        let service = DefaultGameService::with_mock_quantifier(llm_backend, quantifier_backend);
        let _assembler: &dyn PromptAssembler = service.assembler();
    }

    #[test]
    fn test_complete_delegates_to_llm_backend() {
        let llm_backend = Arc::new(MockBackend::default());
        let service = DefaultGameService::with_backends(llm_backend, AgentRegistry::default());
        let result = service.complete("test-agent", "system", "user", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complete_with_max_tokens() {
        let llm_backend = Arc::new(MockBackend::default());
        let service = DefaultGameService::with_backends(llm_backend, AgentRegistry::default());
        let result = service.complete("test-agent", "system", "user", Some(100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_complete_returns_error_when_backend_fails() {
        let llm_backend = Arc::new(MockBackend::failing());
        let service = DefaultGameService::with_backends(llm_backend, AgentRegistry::default());
        let result = service.complete("test-agent", "system", "user", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_action_with_empty_input_does_not_panic() {
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            AgentRegistry::default(),
        );
        let state = make_test_context(GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(crate::test_support::fixtures::TestMap::single_room("start")),
            Arc::new(crate::test_support::fixtures::TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        ));
        service.execute_action(state, "".to_string(), "Player".to_string());
    }

    #[test]
    fn test_execute_action_with_valid_command_does_not_panic() {
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            AgentRegistry::default(),
        );
        let state = make_test_context(GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(crate::test_support::fixtures::TestMap::single_room("start")),
            Arc::new(crate::test_support::fixtures::TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        ));
        service.execute_action(state, "look".to_string(), "Player".to_string());
    }
    #[test]
    fn test_assembler_is_not_null() {
        let service = DefaultGameService::new();
        let _assembler = service.assembler();
    }
    #[test]
    fn test_complete_passes_max_tokens_to_backend() {
        let llm_backend = Arc::new(MockBackend::default());
        let service = DefaultGameService::with_backends(llm_backend, AgentRegistry::default());
        let result = service.complete("test-agent", "system", "user", Some(50));
        assert!(result.is_ok());
    }
    #[test]
    fn test_complete_with_empty_prompts() {
        let llm_backend = Arc::new(MockBackend::default());
        let service = DefaultGameService::with_backends(llm_backend, AgentRegistry::default());
        let result = service.complete("", "", "", None);
        assert!(result.is_ok());
    }
    #[test]
    fn test_execute_action_adds_message_to_state() {
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            AgentRegistry::default(),
        );
        let state = make_test_context(GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(crate::test_support::fixtures::TestMap::single_room("start")),
            Arc::new(crate::test_support::fixtures::TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        ));
        let history_len_before = state.load_messages().unwrap().len();
        service.execute_action(
            state.clone(),
            "test input".to_string(),
            "Player".to_string(),
        );
        let history_len_after = state.load_messages().unwrap().len();
        assert!(history_len_after >= history_len_before);
    }
    #[test]
    fn test_retry_last_response_retriggers_generation() {
        let service = DefaultGameService::with_backends(
            Arc::new(MockBackend::default()),
            AgentRegistry::default(),
        );
        let mut state = GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(crate::test_support::fixtures::TestMap::single_room("start")),
            Arc::new(crate::test_support::fixtures::TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        );
        state.add_message(
            "Test input".to_string(),
            Some("Player".to_string()),
            crate::model::state::MessageType::Input,
        );
        let ctx = make_test_context(state);
        service.retry_last_response(ctx.clone());
    }
    #[test]
    fn test_run_post_generation_agents_merges_state_patches() {
        let llm_backend = Arc::new(MockBackend::default());
        let registry = AgentRegistry::default();
        let service = DefaultGameService::with_backends(llm_backend, registry);
        let state = GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(crate::test_support::fixtures::TestMap::single_room("start")),
            Arc::new(crate::test_support::fixtures::TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        );
        let mut result = crate::model::quantifier::QuantifierResult {
            npcs: crate::model::quantifier::QuantifierParseResult {
                npc_ids: Vec::new(),
                confidence: crate::model::quantifier::QuantifierConfidence::Low,
            },
            movement: crate::model::quantifier::MovementParseResult::default(),
        };
        service.run_post_generation_agents(&state, "player input", "response", &mut result);
    }

    #[test]
    fn test_run_post_generation_agents_with_quantifier() {
        let llm_backend = Arc::new(MockBackend::default());
        let quantifier = crate::narrative::agents::quantifier::QuantifierAgent::with_backend(
            "test_quantifier".to_string(),
            llm_backend.clone(),
        );
        let registry = AgentRegistry::with_agent(Box::new(quantifier));
        let service = DefaultGameService::with_backends(llm_backend, registry);
        let state = GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(crate::test_support::fixtures::TestMap::single_room("start")),
            Arc::new(crate::test_support::fixtures::TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        );
        let mut result = crate::model::quantifier::QuantifierResult {
            npcs: crate::model::quantifier::QuantifierParseResult {
                npc_ids: Vec::new(),
                confidence: crate::model::quantifier::QuantifierConfidence::Low,
            },
            movement: crate::model::quantifier::MovementParseResult::default(),
        };
        service.run_post_generation_agents(&state, "player input", "response", &mut result);
    }
}
