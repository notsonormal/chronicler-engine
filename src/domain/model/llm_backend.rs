//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! LLM backend provider types

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::EngineError;

/// [TRIVIAL_ENUM]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendType {
    #[default]
    OpenRouter,
    DeepSeek,
    Mock,
    Ollama,
}

impl FromStr for LlmBackendType {
    type Err = EngineError;

    /// An unrecognised name is an error; a silent `Mock` fallback would serve
    /// canned text.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openrouter" => Ok(LlmBackendType::OpenRouter),
            "deepseek" => Ok(LlmBackendType::DeepSeek),
            "mock" => Ok(LlmBackendType::Mock),
            "ollama" => Ok(LlmBackendType::Ollama),
            _ => Err(EngineError::Config(format!("Unknown LLM backend '{s}'"))),
        }
    }
}

impl LlmBackendType {
    pub fn api_key_env_var(self) -> Option<&'static str> {
        match self {
            Self::OpenRouter | Self::DeepSeek => Some("OPENROUTER_API_KEY"),
            Self::Ollama | Self::Mock => None,
        }
    }
}
