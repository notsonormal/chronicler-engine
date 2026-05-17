use std::sync::Arc;

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::Connection;
use crate::narrative::agents::quantifier::QuantifierAgent;
use crate::narrative::agents::registry::AgentRegistry;
use crate::storage::llm_message_storage::LlmMessageStorage;

use super::context::GameServiceContext;

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
        Self::with_storage(None)
    }

    pub fn with_storage(storage: Option<Arc<dyn LlmMessageStorage>>) -> Self {
        let settings = crate::settings::load_settings().unwrap_or_default();
        let registry = AgentRegistry::from_configs_with_storage(&settings.agents, storage.clone())
            .unwrap_or_default();
        let connection = settings
            .get_narration_connection()
            .cloned()
            .unwrap_or_else(|| Connection::new("default", "Default", LlmBackendType::Mock));
        let llm_backend = Arc::from(crate::narrative::llm::get_llm_backend_for(
            &connection,
            storage,
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

    /// Convenience constructor for tests that only need a mock quantifier.
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

impl GameService for DefaultGameService {
    fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String) {
        super::actions::execute_action_impl(self, ctx, input, player_name);
    }

    fn retry_last_response(&self, ctx: GameServiceContext) {
        super::retry::retry_last_response_impl(self, ctx);
    }
}
