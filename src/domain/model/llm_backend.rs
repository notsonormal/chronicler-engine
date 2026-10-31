//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM backend provider types

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::EngineError;

/// [TRIVIAL_ENUM]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendType {
    #[default]
    OpenRouter,
    Mock,
    Ollama,
    OpenCodeGo,
}

impl FromStr for LlmBackendType {
    type Err = EngineError;

    /// An unrecognised name is an error; a silent `Mock` fallback would serve
    /// canned text.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|provider| provider.wire_id() == s)
            .ok_or_else(|| EngineError::Config(format!("Unknown LLM backend '{s}'")))
    }
}

impl LlmBackendType {
    pub const ALL: [Self; 4] = [Self::OpenRouter, Self::Mock, Self::Ollama, Self::OpenCodeGo];

    /// The providers the connection form offers, in its order. `Mock` is for
    /// tests only.
    pub const SELECTABLE: [Self; 3] = [Self::OpenRouter, Self::OpenCodeGo, Self::Ollama];

    /// The value the connection form sends; the stored JSON uses the serde name.
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::OpenRouter => "openrouter",
            Self::Mock => "mock",
            Self::Ollama => "ollama",
            Self::OpenCodeGo => "opencode-go",
        }
    }

    /// The name the dashboard shows, and the backend name the LLM message
    /// forensics record.
    pub fn display_label(self) -> &'static str {
        match self {
            Self::OpenRouter => "OpenRouter",
            Self::Mock => "Mock",
            Self::Ollama => "Ollama",
            Self::OpenCodeGo => "OpenCode Go",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::Ollama => "http://localhost:11434/v1",
            Self::OpenCodeGo => "https://opencode.ai/zen/go/v1",
            Self::Mock => "",
        }
    }

    pub fn default_max_context_tokens(self) -> u32 {
        match self {
            Self::OpenRouter | Self::OpenCodeGo => 32768,
            Self::Ollama => 8192,
            Self::Mock => 4096,
        }
    }

    /// The app name sent in `X-Title`, with an `HTTP-Referer`, for the
    /// provider's usage rankings.
    pub fn attribution_title(self) -> Option<&'static str> {
        match self {
            Self::OpenRouter => Some("Chronicler Engine"),
            Self::Ollama | Self::Mock | Self::OpenCodeGo => None,
        }
    }

    pub fn supports_routing(self) -> bool {
        self == Self::OpenRouter
    }

    /// OpenCode Go refuses a call without its session header.
    pub fn session_header(self) -> Option<&'static str> {
        match self {
            Self::OpenCodeGo => Some("x-opencode-session"),
            Self::OpenRouter | Self::Ollama | Self::Mock => None,
        }
    }

    pub fn api_key_env_var(self) -> Option<&'static str> {
        match self {
            Self::OpenRouter => Some("OPENROUTER_API_KEY"),
            Self::OpenCodeGo => Some("OPENCODE_GO_API_KEY"),
            Self::Ollama | Self::Mock => None,
        }
    }
}
