//! [DOC: docs/diataxis/reference/architecture_system.md]
//! Settings and configuration types

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::domain::model::agent::{AgentConfig, Role};
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::prompt_preset::PresetType;
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

    /// The label the dashboard renders for this mode: the mode select's option
    /// text, the allowed-mode checkbox, and the preset badges all derive from it.
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Novel => "Novel",
            Self::InteractiveFiction => "Interactive Fiction",
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

/// How much a model may reason before it answers, in the OpenRouter effort values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// The model does not reason. Serialized as `none`.
    #[serde(rename = "none")]
    Off,
    /// The least reasoning the model accepts.
    Minimal,
    /// Light reasoning.
    Low,
    /// Moderate reasoning.
    Medium,
    /// Heavy reasoning.
    High,
    /// Heavier reasoning than `High`.
    XHigh,
    /// The most reasoning the model accepts.
    Max,
}

impl ReasoningEffort {
    /// Every effort, in the order the connection form offers them.
    pub const ALL: [Self; 7] = [
        Self::Off,
        Self::Minimal,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::XHigh,
        Self::Max,
    ];

    /// The value the provider receives in `reasoning_effort`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
            Self::Max => "max",
        }
    }
}

impl FromStr for ReasoningEffort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|effort| effort.as_str() == s)
            .ok_or_else(|| format!("Unknown reasoning effort: {s}"))
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

impl ModePresetBundle {
    /// `preset_type`'s id in this bundle. `None` for [`PresetType::Options`],
    /// which has no bundle slot.
    pub fn slot(&self, preset_type: PresetType) -> Option<&str> {
        match preset_type {
            PresetType::System => Some(&self.system_prompt_preset_id),
            PresetType::Quantifier => Some(&self.quantifier_prompt_preset_id),
            PresetType::Impersonate => Some(&self.impersonate_prompt_preset_id),
            PresetType::Options => None,
        }
    }

    pub fn slot_mut(&mut self, preset_type: PresetType) -> Option<&mut String> {
        match preset_type {
            PresetType::System => Some(&mut self.system_prompt_preset_id),
            PresetType::Quantifier => Some(&mut self.quantifier_prompt_preset_id),
            PresetType::Impersonate => Some(&mut self.impersonate_prompt_preset_id),
            PresetType::Options => None,
        }
    }
}

impl Default for ModePresetRegistry {
    fn default() -> Self {
        Self(vec![
            settings_defaults::default_bundle_for_mode(NarratorMode::Novel),
            settings_defaults::default_bundle_for_mode(NarratorMode::InteractiveFiction),
        ])
    }
}

impl ModePresetRegistry {
    /// The stored bundle for `mode`, or `None` when the list has no entry for
    /// it.
    pub fn stored_bundle(&self, mode: NarratorMode) -> Option<&ModePresetBundle> {
        self.0.iter().find(|bundle| bundle.mode == mode)
    }

    /// The bundle for `mode`, or the constructed default when the list has
    /// no entry for it.
    pub fn bundle_for(&self, mode: NarratorMode) -> ModePresetBundle {
        self.stored_bundle(mode)
            .cloned()
            .unwrap_or_else(|| settings_defaults::default_bundle_for_mode(mode))
    }

    pub fn set_bundle(&mut self, bundle: ModePresetBundle) {
        match self.0.iter_mut().find(|b| b.mode == bundle.mode) {
            Some(slot) => *slot = bundle,
            None => self.0.push(bundle),
        }
    }

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

impl TextCheckSettings {
    /// Disabled mode can never carry auto-check, so the invariant lives in
    /// [`Self::normalised_auto_check`]: a legacy row storing the inconsistent
    /// pair (`mode: Disabled` with `enable_auto_check: true`) still reads false,
    /// and the write path normalises through the same rule.
    pub fn effective_enable_auto_check(&self) -> bool {
        self.normalised_auto_check(self.enable_auto_check)
    }

    pub fn set_mode_and_auto_check(&mut self, mode: TextCheckMode, requested_auto_check: bool) {
        self.mode = mode;
        self.enable_auto_check = self.normalised_auto_check(requested_auto_check);
    }

    fn normalised_auto_check(&self, requested_auto_check: bool) -> bool {
        requested_auto_check && !matches!(self.mode, TextCheckMode::Disabled)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConnection {
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
    pub max_tokens_ceiling: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_context_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// The upstream hosts a routing provider tries, in order. Empty lets the
    /// provider choose.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_order: Vec<String>,
    /// Use only the hosts of `provider_order`, with no fallback to others.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pin_provider_order: bool,
}

pub const DEFAULT_MAX_TOKENS: u32 = 2048;

impl LlmConnection {
    pub fn max_tokens_for(&self, preset_value: Option<u32>) -> u32 {
        let requested = preset_value.unwrap_or(DEFAULT_MAX_TOKENS);
        self.max_tokens_ceiling
            .map_or(requested, |ceiling| requested.min(ceiling))
    }

    pub fn new(id: impl Into<String>, name: impl Into<String>, provider: LlmBackendType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider,
            model: "openai/gpt-4o-mini".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens_ceiling: None,
            max_context_tokens: None,
            reasoning_effort: None,
            provider_order: Vec::new(),
            pin_provider_order: false,
        }
    }

    /// The key this connection authenticates with: its own, else its provider's
    /// environment fallback. A blank value on either side counts as absent, so a
    /// stray empty string cannot shadow the fallback.
    pub fn resolve_api_key(&self) -> Option<String> {
        self.resolve_api_key_with(|name| std::env::var(name).ok())
    }

    pub fn resolve_api_key_with(&self, lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
        let non_blank = |key: String| (!key.trim().is_empty()).then_some(key);
        self.api_key.clone().and_then(non_blank).or_else(|| {
            self.provider
                .api_key_env_var()
                .and_then(lookup)
                .and_then(non_blank)
        })
    }

    pub fn check_api_key_available(&self) -> Result<(), EngineError> {
        self.check_api_key_available_with(|name| std::env::var(name).ok())
    }

    pub fn check_api_key_available_with(
        &self,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<(), EngineError> {
        let Some(env_var) = self.provider.api_key_env_var() else {
            return Ok(());
        };
        if self.resolve_api_key_with(lookup).is_some() {
            return Ok(());
        }
        let label = if self.name.trim().is_empty() {
            &self.id
        } else {
            &self.name
        };
        Err(EngineError::Config(format!(
            "{} connection '{label}' has no API key: set one on the connection \
             in Settings, or export {env_var}",
            self.provider.display_label(),
        )))
    }

    pub fn resolve_base_url(&self) -> String {
        if let Some(url) = &self.base_url {
            return url.clone();
        }
        self.provider.default_base_url().to_string()
    }

    pub fn resolve_max_context_tokens(&self) -> u32 {
        self.max_context_tokens
            .unwrap_or(self.provider.default_max_context_tokens())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub connections: Vec<LlmConnection>,
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    /// Its own Connection; `None` uses the Narration Connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impersonate_connection_id: Option<String>,
    /// Its own Connection; `None` uses the Narration Connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options_connection_id: Option<String>,
    #[serde(default)]
    pub text_check: TextCheckSettings,
    #[serde(default = "AgentConfig::defaults")]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub mode_preset_registry: ModePresetRegistry,
    #[serde(default = "settings_defaults::default_active_options_prompt_preset_id")]
    pub active_options_prompt_preset_id: String,
}

/// Why `AppSettings::remove_connection` refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoveConnectionError {
    /// No stored Connection has the id.
    NotFound,
    /// The settings keep at least one Connection.
    LastConnection,
    /// The Roles whose required slot names the Connection.
    InUse(Vec<Role>),
}

enum SlotKind {
    /// The Role that a refused removal names.
    Required(Role),
    /// An empty slot uses the Narration Connection. A removal clears it.
    Optional(fn(&mut AppSettings) -> &mut Option<String>),
}

struct ConnectionSlot {
    preset_type: PresetType,
    field: &'static str,
    /// `None` for an empty optional slot.
    id: fn(&AppSettings) -> Option<&str>,
    kind: SlotKind,
}

/// The one table of the Connection slots. `PresetType` is the slot key
/// (`AppSettings::connection_for`); the Python scripts read the same four fields
/// (`scripts/validate_data.py`, `scripts/trial.py`).
const CONNECTION_SLOTS: [ConnectionSlot; 4] = [
    ConnectionSlot {
        preset_type: PresetType::System,
        field: "narration_connection_id",
        id: |settings| Some(settings.narration_connection_id.as_str()),
        kind: SlotKind::Required(Role::Narrator),
    },
    ConnectionSlot {
        preset_type: PresetType::Impersonate,
        field: "impersonate_connection_id",
        id: |settings| settings.impersonate_connection_id.as_deref(),
        kind: SlotKind::Optional(|settings| &mut settings.impersonate_connection_id),
    },
    ConnectionSlot {
        preset_type: PresetType::Quantifier,
        field: "quantifier_connection_id",
        id: |settings| Some(settings.quantifier_connection_id.as_str()),
        kind: SlotKind::Required(Role::Quantifier),
    },
    ConnectionSlot {
        preset_type: PresetType::Options,
        field: "options_connection_id",
        id: |settings| settings.options_connection_id.as_deref(),
        kind: SlotKind::Optional(|settings| &mut settings.options_connection_id),
    },
];

impl Default for AppSettings {
    fn default() -> Self {
        let gpt4o = LlmConnection::new(
            "openrouter-gpt-4o-mini",
            "openrouter-gpt-4o-mini",
            LlmBackendType::OpenRouter,
        );
        let euryale = LlmConnection {
            model: "sao10k/l3.3-euryale-70b".into(),
            ..LlmConnection::new(
                "openrouter-euryale",
                "openrouter-euryale",
                LlmBackendType::OpenRouter,
            )
        };
        let gemma = LlmConnection {
            model: "hf.co/mradermacher/gemma-4-26B-A4B-it-abliterated-i1-GGUF:latest".into(),
            base_url: Some("http://localhost:11434/v1".into()),
            ..LlmConnection::new(
                "ollama-gemma-4-26B",
                "ollama-gemma-4-26B",
                LlmBackendType::Ollama,
            )
        };
        Self {
            connections: vec![gpt4o, euryale, gemma],
            narration_connection_id: "openrouter-gpt-4o-mini".into(),
            quantifier_connection_id: "openrouter-gpt-4o-mini".into(),
            impersonate_connection_id: None,
            options_connection_id: None,
            text_check: TextCheckSettings::default(),
            agents: AgentConfig::defaults(),
            mode_preset_registry: ModePresetRegistry::default(),
            active_options_prompt_preset_id:
                settings_defaults::default_active_options_prompt_preset_id(),
        }
    }
}

impl AppSettings {
    /// The id the active slot for `preset_type` holds in `mode`. Options presets
    /// are mode-agnostic, so `mode` is ignored for [`PresetType::Options`].
    pub fn active_preset_id(&self, preset_type: PresetType, mode: NarratorMode) -> &str {
        if !preset_type.is_mode_tagged() {
            return &self.active_options_prompt_preset_id;
        }
        self.mode_preset_registry
            .stored_bundle(mode)
            .and_then(|bundle| bundle.slot(preset_type))
            .unwrap_or_else(|| settings_defaults::default_preset_id(preset_type, mode))
    }

    /// Points `preset_type`'s active slot in `mode` at `id`. Options presets are
    /// mode-agnostic, so they write the settings-level default instead.
    pub fn set_active_preset(&mut self, preset_type: PresetType, mode: NarratorMode, id: String) {
        if !preset_type.is_mode_tagged() {
            self.active_options_prompt_preset_id = id;
            return;
        }
        let mut bundle = self.mode_preset_registry.bundle_for(mode);
        if let Some(slot) = bundle.slot_mut(preset_type) {
            *slot = id;
            self.mode_preset_registry.set_bundle(bundle);
        }
    }

    pub fn find_connection(&self, id: &str) -> Option<&LlmConnection> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn find_connection_mut(&mut self, id: &str) -> Option<&mut LlmConnection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    pub fn connection_for(&self, preset_type: PresetType) -> crate::error::Result<LlmConnection> {
        let (field, id) = self.slot_reference(preset_type);
        self.find_connection(id).cloned().ok_or_else(|| {
            EngineError::Config(format!("{field} '{id}' is not in the connections list"))
        })
    }

    /// The slot that serves `preset_type`: its own field and id, or the Narration
    /// slot while an optional slot is empty.
    fn slot_reference(&self, preset_type: PresetType) -> (&'static str, &str) {
        CONNECTION_SLOTS
            .iter()
            .find(|slot| slot.preset_type == preset_type)
            .and_then(|slot| (slot.id)(self).map(|id| (slot.field, id)))
            .unwrap_or(("narration_connection_id", &self.narration_connection_id))
    }

    /// An empty optional slot is no reference.
    pub fn check_connection_slots(&self) -> crate::error::Result<()> {
        for slot in &CONNECTION_SLOTS {
            self.connection_for(slot.preset_type)?;
        }
        Ok(())
    }

    /// Removes the Connection `id`. Every refusal comes before the first change,
    /// so a refused removal leaves the settings as they were.
    pub fn remove_connection(&mut self, id: &str) -> Result<(), RemoveConnectionError> {
        let Some(index) = self.connections.iter().position(|c| c.id == id) else {
            return Err(RemoveConnectionError::NotFound);
        };
        if self.connections.len() <= 1 {
            return Err(RemoveConnectionError::LastConnection);
        }
        let roles: Vec<Role> = CONNECTION_SLOTS
            .iter()
            .filter_map(|slot| match slot.kind {
                SlotKind::Required(role) if (slot.id)(self) == Some(id) => Some(role),
                _ => None,
            })
            .collect();
        if !roles.is_empty() {
            return Err(RemoveConnectionError::InUse(roles));
        }
        self.clear_optional_slots_naming(id);
        self.connections.remove(index);
        Ok(())
    }

    fn clear_optional_slots_naming(&mut self, connection_id: &str) {
        for slot in &CONNECTION_SLOTS {
            if let SlotKind::Optional(slot_mut) = slot.kind
                && (slot.id)(self) == Some(connection_id)
            {
                *slot_mut(self) = None;
            }
        }
    }
}
