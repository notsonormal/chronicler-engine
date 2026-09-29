//! [DOC: docs/diataxis/reference/architecture_system.md]
//! Settings and configuration types

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::domain::model::agent::AgentConfig;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::utils::settings_defaults;
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TextCheckMode {
    /// Text checking disabled; `check` short-circuits with `None`.
    #[default]
    Disabled,
    /// Spell-check only.
    Spell,
    /// Grammar-check only.
    Grammar,
    /// Spell and grammar checks both applied.
    SpellGrammar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NarratorMode {
    /// Novel/RP posture — default.
    #[default]
    Novel,
    /// Interactive-fiction/CYOA posture (second-person narration of commanded actions).
    InteractiveFiction,
}

impl NarratorMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Novel => "novel",
            Self::InteractiveFiction => "interactive_fiction",
        }
    }

    pub fn parse_or_default(s: &str) -> Self {
        Self::from_str(s).unwrap_or_else(|e| {
            tracing::warn!("Invalid narrator mode '{s}', falling back to Novel: {e}");
            Self::Novel
        })
    }

    /// The perspective a fresh game of this mode starts at. Also the nudge
    /// target when a game switches into this mode while sitting at the other
    /// mode's default perspective.
    pub fn default_perspective(&self) -> NarrativePerspective {
        match self {
            Self::Novel => NarrativePerspective::Third,
            Self::InteractiveFiction => NarrativePerspective::Second,
        }
    }
}

impl FromStr for NarratorMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "novel" => Ok(Self::Novel),
            "interactive_fiction" => Ok(Self::InteractiveFiction),
            _ => Err(format!("Unknown narrator mode: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModePresetBundle {
    #[serde(default = "settings_defaults::default_narrator_mode")]
    pub mode: NarratorMode,
    #[serde(default = "settings_defaults::default_active_system_prompt_preset_id")]
    pub system_prompt_preset_id: String,
    #[serde(default = "settings_defaults::default_active_quantifier_prompt_preset_id")]
    pub quantifier_prompt_preset_id: String,
    #[serde(default = "settings_defaults::default_active_impersonate_prompt_preset_id")]
    pub impersonate_prompt_preset_id: String,
}

/// Mode-tagged JSON list of per-mode default preset bundles. The list may be
/// partial (or empty) — `bundle_for` falls back to constructed defaults for a
/// missing entry, so a missing mode never fails a lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModePresetRegistry(pub Vec<ModePresetBundle>);

impl Default for ModePresetRegistry {
    fn default() -> Self {
        Self(vec![
            settings_defaults::default_bundle_for_mode(NarratorMode::Novel),
            settings_defaults::default_bundle_for_mode(NarratorMode::InteractiveFiction),
        ])
    }
}

impl ModePresetRegistry {
    /// The bundle for `mode`, or the constructed default when the list has
    /// no entry for it.
    pub fn bundle_for(&self, mode: NarratorMode) -> ModePresetBundle {
        self.0
            .iter()
            .find(|b| b.mode == mode)
            .cloned()
            .unwrap_or_else(|| settings_defaults::default_bundle_for_mode(mode))
    }

    /// Insert or replace the entry for `bundle.mode`.
    pub fn set_bundle(&mut self, bundle: ModePresetBundle) {
        match self.0.iter_mut().find(|b| b.mode == bundle.mode) {
            Some(slot) => *slot = bundle,
            None => self.0.push(bundle),
        }
    }

    /// Whether any mode's bundle defaults to the preset `id`.
    pub fn references(&self, id: &str) -> bool {
        self.0.iter().any(|b| {
            b.system_prompt_preset_id == id
                || b.quantifier_prompt_preset_id == id
                || b.impersonate_prompt_preset_id == id
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NarrativePerspective {
    /// Second person ("you walked") — classic interactive-fiction/CYOA voice.
    Second,
    /// Third person ("she walked") — default novel/RP voice.
    #[default]
    Third,
}

impl NarrativePerspective {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Third => "third",
        }
    }

    pub fn parse_or_default(s: &str) -> Self {
        Self::from_str(s).unwrap_or_else(|e| {
            tracing::warn!("Invalid narrative perspective '{s}', falling back to Third: {e}");
            Self::Third
        })
    }
}

impl FromStr for NarrativePerspective {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "second" => Ok(Self::Second),
            "third" => Ok(Self::Third),
            _ => Err(format!("Unknown narrative perspective: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NarrativeTense {
    /// Present tense ("she walks").
    Present,
    /// Past tense ("she walked") — default.
    #[default]
    Past,
}

impl NarrativeTense {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Past => "past",
        }
    }

    pub fn parse_or_default(s: &str) -> Self {
        Self::from_str(s).unwrap_or_else(|e| {
            tracing::warn!("Invalid narrative tense '{s}', falling back to Past: {e}");
            Self::Past
        })
    }
}

impl FromStr for NarrativeTense {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "present" => Ok(Self::Present),
            "past" => Ok(Self::Past),
            _ => Err(format!("Unknown narrative tense: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextCheckSettings {
    pub mode: TextCheckMode,
    #[serde(default = "settings_defaults::default_enable_auto_check")]
    pub enable_auto_check: bool,
    #[serde(default)]
    pub ignored_words: Vec<String>,
}

impl Default for TextCheckSettings {
    fn default() -> Self {
        Self {
            mode: TextCheckMode::default(),
            enable_auto_check: settings_defaults::default_enable_auto_check(),
            ignored_words: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmProviderConfig {
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

impl LlmProviderConfig {
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

    /// The key this connection authenticates with: its own, else its provider's
    /// environment fallback. A blank value on either side counts as absent, so a
    /// stray empty string cannot shadow the fallback.
    pub fn resolve_api_key(&self) -> Option<String> {
        let non_blank = |key: String| (!key.trim().is_empty()).then_some(key);
        self.api_key.clone().and_then(non_blank).or_else(|| {
            self.provider
                .api_key_env_var()
                .and_then(|name| std::env::var(name).ok())
                .and_then(non_blank)
        })
    }

    pub fn check_api_key_available(&self) -> Result<(), EngineError> {
        // Delete this guard once the DeepSeek backend lands: its stub currently
        // reports a more truthful not-implemented error than a missing key would.
        if self.provider == LlmBackendType::DeepSeek {
            return Ok(());
        }
        let Some(env_var) = self.provider.api_key_env_var() else {
            return Ok(());
        };
        if self.resolve_api_key().is_some() {
            return Ok(());
        }
        let label = if self.name.trim().is_empty() {
            &self.id
        } else {
            &self.name
        };
        Err(EngineError::Config(format!(
            "{:?} connection '{label}' has no API key: set one on the connection \
             in Settings, or export {env_var}",
            self.provider,
        )))
    }

    pub fn resolve_base_url(&self) -> String {
        if let Some(url) = &self.base_url {
            return url.clone();
        }
        match self.provider {
            LlmBackendType::Ollama => settings_defaults::default_ollama_base_url(),
            LlmBackendType::OpenRouter | LlmBackendType::DeepSeek => {
                "https://openrouter.ai/api/v1".into()
            }
            LlmBackendType::Mock => String::new(),
        }
    }

    pub fn resolve_max_context_tokens(&self) -> u32 {
        self.max_context_tokens.unwrap_or(match self.provider {
            LlmBackendType::Ollama => 8192,
            LlmBackendType::OpenRouter | LlmBackendType::DeepSeek => 32768,
            LlmBackendType::Mock => 4096,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub connections: Vec<LlmProviderConfig>,
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    #[serde(default = "settings_defaults::default_response_length")]
    pub response_length: String,
    #[serde(default)]
    pub text_check: TextCheckSettings,
    #[serde(default = "AgentConfig::defaults")]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub mode_preset_registry: ModePresetRegistry,
    #[serde(default = "settings_defaults::default_active_options_prompt_preset_id")]
    pub active_options_prompt_preset_id: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let gpt4o = LlmProviderConfig {
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
        let euryale = LlmProviderConfig {
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
        let gemma = LlmProviderConfig {
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
            response_length: settings_defaults::default_response_length(),
            text_check: TextCheckSettings::default(),
            agents: AgentConfig::defaults(),
            mode_preset_registry: ModePresetRegistry::default(),
            active_options_prompt_preset_id:
                settings_defaults::default_active_options_prompt_preset_id(),
        }
    }
}

impl AppSettings {
    pub fn find_connection(&self, id: &str) -> Option<&LlmProviderConfig> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn find_connection_mut(&mut self, id: &str) -> Option<&mut LlmProviderConfig> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    pub fn get_narration_connection(&self) -> Option<&LlmProviderConfig> {
        self.find_connection(&self.narration_connection_id)
    }

    pub fn get_quantifier_connection(&self) -> Option<&LlmProviderConfig> {
        self.find_connection(&self.quantifier_connection_id)
    }

    /// The narration connection, or an error naming the missing id.
    pub fn narration_connection(&self) -> crate::error::Result<LlmProviderConfig> {
        self.get_narration_connection().cloned().ok_or_else(|| {
            EngineError::Config(format!(
                "narration_connection_id '{}' is not in the connections list",
                self.narration_connection_id
            ))
        })
    }

    /// The quantifier connection, or an error naming the missing id.
    pub fn quantifier_connection(&self) -> crate::error::Result<LlmProviderConfig> {
        self.get_quantifier_connection().cloned().ok_or_else(|| {
            EngineError::Config(format!(
                "quantifier_connection_id '{}' is not in the connections list",
                self.quantifier_connection_id
            ))
        })
    }
}
