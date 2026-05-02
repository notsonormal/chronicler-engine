use serde::{Deserialize, Serialize};

use crate::model::llm_backend::LlmBackendType;

fn default_ollama_base_url() -> String {
    "http://localhost:11434/v1".into()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_backend: LlmBackendType,
    pub llm_model: String,
    pub quantifier_model: String,
    pub openrouter_api_key: Option<String>,
    #[serde(default)]
    pub quantifier_backend: LlmBackendType,
    #[serde(default = "default_ollama_base_url")]
    pub ollama_base_url: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_backend: LlmBackendType::OpenRouter,
            llm_model: "openai/gpt-4o-mini".into(),
            quantifier_model: "openai/gpt-4o-mini".into(),
            openrouter_api_key: None,
            quantifier_backend: LlmBackendType::OpenRouter,
            ollama_base_url: default_ollama_base_url(),
        }
    }
}
