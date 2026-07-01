//! [DOC: docs/system/llm_processing.md]
//! LLM factory - wires LlmProvider port to provider impls and returns LlmCallRecorder

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::settings::Connection;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::adapters::driven::storage::Storage;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::ports::llm_message_repository::LlmMessageRepository;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::llm::providers::{
    OpenRouterBackend, DeepSeekBackend, OllamaBackend, MockBackend,
};

/// Create an LlmCallRecorder for the given connection.
/// This is the composition root for LLM providers - wires the provider impl to the orchestrator.
pub fn get_llm_recorder_for(
    connection: &Connection,
    storage: Arc<Storage>,
) -> Result<Arc<LlmCallRecorder>, EngineError> {
    tracing::info!(
        "Creating LLM recorder: provider={:?}, model={}",
        connection.provider,
        connection.model
    );

    // Create the provider (adapter)
    let provider: Arc<dyn LlmProvider> = match connection.provider {
        LlmBackendType::Mock => Arc::new(MockBackend::new()),
        LlmBackendType::DeepSeek => Arc::new(DeepSeekBackend::from_connection(connection)),
        LlmBackendType::OpenRouter => Arc::new(OpenRouterBackend::from_connection(connection)),
        LlmBackendType::Ollama => Arc::new(OllamaBackend::from_connection(connection)),
    };

    // Storage implements LlmMessageRepository - use it as the forensics repository
    let forensics: Arc<dyn LlmMessageRepository> = storage;

    // Return the orchestrator
    Ok(Arc::new(LlmCallRecorder::new(provider, forensics)))
}
