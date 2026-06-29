//! [DOC: docs/system/llm_processing.md]
//! LLM backend abstraction

use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::settings::Connection;
use crate::adapters::driven::llm::transport::ChatCompletionResult;
use crate::adapters::driven::storage::Storage;

pub const AGENT_NARRATOR: &str = "narrator";
pub const AGENT_QUANTIFIER: &str = "quantifier";
pub const AGENT_TRIGGER: &str = "trigger";
pub const AGENT_DIALOGUE: &str = "dialogue";

#[derive(Debug)]
pub struct LlmCallResult {
    pub text: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub backend_name: String,
    pub model_name: String,
    pub agent_name: String,
}

impl LlmCallResult {
    pub fn from_chat_result(
        agent_name: &str,
        backend_name: &str,
        model_name: &str,
        chat: ChatCompletionResult,
    ) -> Self {
        Self {
            text: chat.text,
            system_prompt: chat.system_prompt,
            user_prompt: chat.user_prompt,
            raw_request_json: chat.raw_request_json,
            raw_response_json: chat.raw_response_json,
            backend_name: backend_name.to_string(),
            model_name: model_name.to_string(),
            agent_name: agent_name.to_string(),
        }
    }

    pub fn to_message(&self) -> LlmMessage {
        LlmMessage {
            id: 0,
            agent_name: self.agent_name.clone(),
            backend_name: self.backend_name.clone(),
            model_name: self.model_name.clone(),
            system_prompt: self.system_prompt.clone(),
            user_prompt: self.user_prompt.clone(),
            raw_request_json: self.raw_request_json.clone(),
            raw_response_json: self.raw_response_json.clone(),
            parsed_response: self.text.clone(),
            error_message: None,
            created_at: chrono::Utc::now(),
        }
    }
}

pub trait LlmBackend: Send + Sync {
    fn model(&self) -> &str;
    fn name(&self) -> &str;
    fn storage(&self) -> Option<&Arc<Storage>> {
        None
    }

    fn save_message(&self, message: &LlmMessage) {
        if let Some(storage) = self.storage() {
            let _ = storage.save_llm_message(message);
        }
    }

    fn postprocess_response_text(&self, text: &str) -> String {
        crate::adapters::driven::llm::providers::sanitize::sanitize_llm_output(text)
    }

    fn wrap_and_save(&self, agent_name: &str, mut chat: ChatCompletionResult) -> LlmCallResult {
        chat.text = self.postprocess_response_text(&chat.text);
        let result = LlmCallResult::from_chat_result(agent_name, self.name(), self.model(), chat);
        self.save_message(&result.to_message());
        result
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError>;
}

pub use crate::domain::model::llm_backend::LlmBackendType;

pub fn get_llm_backend_for(
    connection: &Connection,
    storage: Option<Arc<Storage>>,
) -> Box<dyn LlmBackend> {
    tracing::info!(
        "Creating LLM backend: provider={:?}, model={}",
        connection.provider,
        connection.model
    );
    match connection.provider {
        LlmBackendType::Mock => Box::new(
            crate::adapters::driven::llm::providers::MockBackend::new(storage),
        ),
        LlmBackendType::DeepSeek => Box::new(
            crate::adapters::driven::llm::providers::DeepSeekBackend::from_connection(
                connection, storage,
            ),
        ),
        LlmBackendType::OpenRouter => Box::new(
            crate::adapters::driven::llm::providers::OpenRouterBackend::from_connection(
                connection, storage,
            ),
        ),
        LlmBackendType::Ollama => Box::new(
            crate::adapters::driven::llm::providers::OllamaBackend::from_connection(
                connection, storage,
            ),
        ),
    }
}

/// Merge system + user for models that ignore system role.
pub fn merge_single_user_message(system_prompt: &str, user_text: &str) -> String {
    format!("[SYSTEM]\n{system_prompt}\n\n{user_text}")
}
