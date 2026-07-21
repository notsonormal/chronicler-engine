//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! LLM factory - wires LlmProvider port to provider impls and returns LlmCallRecorder

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::settings::LlmProviderConfig;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::adapters::driven::storage::Storage;
use crate::application::llm_message::{LlmMessage, SaveLlmMessageFn};
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::llm::providers::{
    OpenRouterBackend, DeepSeekBackend, OllamaBackend, MockBackend,
};

/// Create an LlmCallRecorder for the given connection.
/// This is the composition root for LLM providers - wires the provider impl to the orchestrator.
pub fn get_llm_recorder_for(
    connection: &LlmProviderConfig,
    storage: Arc<Storage>,
) -> Result<Arc<LlmCallRecorder>, EngineError> {
    tracing::info!(
        "Creating LLM recorder: provider={:?}, model={}",
        connection.provider,
        connection.model
    );

    let provider: Arc<dyn LlmProvider> = match connection.provider {
        LlmBackendType::Mock => Arc::new(MockBackend::new()),
        LlmBackendType::DeepSeek => Arc::new(DeepSeekBackend::from_config(connection)),
        LlmBackendType::OpenRouter => Arc::new(OpenRouterBackend::from_config(connection)),
        LlmBackendType::Ollama => Arc::new(OllamaBackend::from_config(connection)),
    };

    let save_fn: SaveLlmMessageFn =
        Arc::new(move |message: &LlmMessage| storage.save_llm_message(message));

    Ok(Arc::new(LlmCallRecorder::new(provider, save_fn)))
}
