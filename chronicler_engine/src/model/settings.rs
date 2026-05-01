use serde::{Deserialize, Serialize};

use crate::narrative::llm::LlmBackendType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_backend: LlmBackendType,
    pub llm_model: String,
    pub quantifier_model: String,
    pub openrouter_api_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_backend: LlmBackendType::OpenRouter,
            llm_model: "openai/gpt-4o-mini".into(),
            quantifier_model: "openai/gpt-4o-mini".into(),
            openrouter_api_key: None,
        }
    }
}
