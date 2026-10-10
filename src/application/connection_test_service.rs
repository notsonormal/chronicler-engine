//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Connection test service — one short provider call that records nothing.
//!
//! The test must not pass through `LlmCallRecorder`, which persists every
//! attempt: a row would move the role's health and the failure banner, and a
//! short probe can pass where a real call fails. The provider is built from
//! the connection config instead, so the call leaves no forensic row.

use std::sync::Arc;
use std::time::Instant;

use crate::application::errors::ApplicationError;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::settings_service::SettingsService;
use crate::domain::model::settings::LlmProviderConfig;
use crate::error::{EngineError, LlmFailure};

pub type ProviderFactory = Arc<dyn Fn(&LlmProviderConfig) -> Arc<dyn LlmProvider> + Send + Sync>;

/// The test's agent label. It is not a role, so the call never lands in
/// `role_health`, which is keyed by the real agent names.
const TEST_AGENT_NAME: &str = "connection_test";
const TEST_SYSTEM_PROMPT: &str = "You are a connectivity probe. Answer with one short word.";
const TEST_USER_PROMPT: &str = "Reply with one short word to confirm you are reachable.";
const TEST_MAX_TOKENS: u32 = 8;

#[derive(Debug)]
pub struct ConnectionTestResult {
    pub elapsed_ms: u128,
    pub backend_name: String,
    pub model_name: String,
}

#[derive(Clone)]
pub struct ConnectionTestService {
    settings_service: SettingsService,
    provider_factory: ProviderFactory,
}

impl ConnectionTestService {
    pub fn new(settings_service: SettingsService, provider_factory: ProviderFactory) -> Self {
        Self {
            settings_service,
            provider_factory,
        }
    }

    pub fn test_saved_connection(
        &self,
        connection_id: &str,
    ) -> Result<ConnectionTestResult, ApplicationError> {
        let settings = self.settings_service.get_settings()?;
        let connection = settings
            .find_connection(connection_id)
            .ok_or_else(|| ApplicationError::validation("Connection not found"))?
            .clone();
        self.test_connection(&connection)
    }

    pub fn test_connection(
        &self,
        connection: &LlmProviderConfig,
    ) -> Result<ConnectionTestResult, ApplicationError> {
        connection.check_api_key_available()?;
        let provider = (self.provider_factory)(connection);
        let started = Instant::now();
        let (backend_name, model_name) = match provider.complete(
            TEST_AGENT_NAME,
            TEST_SYSTEM_PROMPT,
            TEST_USER_PROMPT,
            Some(TEST_MAX_TOKENS),
        ) {
            Ok(result) => (result.backend_name, result.model_name),
            // The probe's budget is tiny, so a reasoning model can spend it all before
            // answering. The provider still answered, which is all the test checks.
            Err(EngineError::Llm(LlmFailure::TokenBudgetSpent { .. })) => {
                (provider.name().to_string(), provider.model().to_string())
            }
            Err(error) => return Err(error.into()),
        };
        Ok(ConnectionTestResult {
            elapsed_ms: started.elapsed().as_millis(),
            backend_name,
            model_name,
        })
    }

    #[cfg(feature = "testing")]
    pub fn with_provider_factory(self, provider_factory: ProviderFactory) -> Self {
        Self {
            provider_factory,
            ..self
        }
    }
}
