use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendType {
    OpenRouter,
    DeepSeek,
    Mock,
}

impl From<&str> for LlmBackendType {
    fn from(s: &str) -> Self {
        match s {
            "deepseek" => LlmBackendType::DeepSeek,
            "mock" => LlmBackendType::Mock,
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
