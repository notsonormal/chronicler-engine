use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendType {
    OpenRouter,
    DeepSeek,
    Mock,
}

impl LlmBackendType {
    pub fn from_env() -> Self {
        match std::env::var("LLM_BACKEND").as_deref() {
            Ok("deepseek") => LlmBackendType::DeepSeek,
            Ok("mock") => LlmBackendType::Mock,
            _ => LlmBackendType::OpenRouter,
        }
    }
}
