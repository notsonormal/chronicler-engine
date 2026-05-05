use serde::{Deserialize, Serialize};

use crate::model::llm_backend::LlmBackendType;

fn default_ollama_base_url() -> String {
    "http://localhost:11434/v1".into()
}

/// [DOC: docs/architecture/system.md]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub provider: LlmBackendType,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default)]
    pub single_user_message: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_context_tokens: Option<u32>,
}

impl Connection {
    pub fn new(id: impl Into<String>, name: impl Into<String>, provider: LlmBackendType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider,
            model: "openai/gpt-4o-mini".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        }
    }

    /// Resolve the API key for this connection.
    /// Returns the connection's own key if set, otherwise falls back to
    /// the provider-specific environment variable.
    pub fn resolve_api_key(&self) -> Option<String> {
        if self.api_key.is_some() {
            return self.api_key.clone();
        }
        match self.provider {
            LlmBackendType::OpenRouter | LlmBackendType::DeepSeek => {
                std::env::var("OPENROUTER_API_KEY").ok()
            }
            LlmBackendType::Ollama | LlmBackendType::Mock => None,
        }
    }

    /// Resolve the base URL for this connection.
    pub fn resolve_base_url(&self) -> String {
        if let Some(url) = &self.base_url {
            return url.clone();
        }
        match self.provider {
            LlmBackendType::Ollama => {
                std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| default_ollama_base_url())
            }
            LlmBackendType::OpenRouter | LlmBackendType::DeepSeek => {
                "https://openrouter.ai/api/v1".into()
            }
            LlmBackendType::Mock => String::new(),
        }
    }

    /// Resolve the context window size for this connection.
    /// Returns 8192 for Ollama (local models typically have smaller contexts),
    /// 32768 for API models (OpenRouter, DeepSeek) when unset.
    pub fn resolve_max_context_tokens(&self) -> u32 {
        self.max_context_tokens.unwrap_or(match self.provider {
            LlmBackendType::Ollama => 8192,
            LlmBackendType::OpenRouter | LlmBackendType::DeepSeek => 32768,
            LlmBackendType::Mock => 4096,
        })
    }
}

/// [DOC: docs/architecture/system.md]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub connections: Vec<Connection>,
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    #[serde(default = "default_response_length")]
    pub response_length: String,
}

fn default_response_length() -> String {
    "flexible, based on the current scene. During a conversation, keep it concise (under 150 words) to allow back-and-forth. For scene transitions, travel, or plot developments, build content (above 150 words), but allow the player to react.".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        let gpt4o = Connection {
            id: "openrouter-gpt-4o-mini".into(),
            name: "openrouter-gpt-4o-mini".into(),
            provider: LlmBackendType::OpenRouter,
            model: "openai/gpt-4o-mini".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        };
        let euryale = Connection {
            id: "openrouter-euryale".into(),
            name: "openrouter-euryale".into(),
            provider: LlmBackendType::OpenRouter,
            model: "sao10k/l3.3-euryale-70b".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        };
        let gemma = Connection {
            id: "ollama-gemma-4-26B".into(),
            name: "ollama-gemma-4-26B".into(),
            provider: LlmBackendType::Ollama,
            model: "hf.co/mradermacher/gemma-4-26B-A4B-it-abliterated-i1-GGUF:latest".into(),
            api_key: None,
            base_url: Some("http://localhost:11434/v1".into()),
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        };
        Self {
            connections: vec![gpt4o, euryale, gemma],
            narration_connection_id: "openrouter-gpt-4o-mini".into(),
            quantifier_connection_id: "openrouter-gpt-4o-mini".into(),
            response_length: default_response_length(),
        }
    }
}

impl AppSettings {
    pub fn find_connection(&self, id: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn find_connection_mut(&mut self, id: &str) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    pub fn get_narration_connection(&self) -> Option<&Connection> {
        self.find_connection(&self.narration_connection_id)
    }

    pub fn get_quantifier_connection(&self) -> Option<&Connection> {
        self.find_connection(&self.quantifier_connection_id)
    }
}
