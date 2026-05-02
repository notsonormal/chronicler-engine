use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendType {
    #[default]
    OpenRouter,
    DeepSeek,
    Mock,
    Ollama,
}

impl From<&str> for LlmBackendType {
    fn from(s: &str) -> Self {
        match s {
            "deepseek" => LlmBackendType::DeepSeek,
            "mock" => LlmBackendType::Mock,
            "ollama" => LlmBackendType::Ollama,
            _ => LlmBackendType::OpenRouter,
        }
    }
}

impl LlmBackendType {
    pub fn from_env() -> Self {
        std::env::var("LLM_BACKEND")
            .as_deref()
            .map_or(LlmBackendType::OpenRouter, Self::from)
    }
}
