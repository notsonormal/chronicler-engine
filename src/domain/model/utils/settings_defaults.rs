//! [DOC: docs/diataxis/reference/architecture_system.md]
//! Serde default-fn-pointers for `AppSettings` fields. Cannot become methods — `#[serde(default = "...")]` requires a fn path.

pub fn default_enable_auto_check() -> bool {
    true
}

pub fn default_ollama_base_url() -> String {
    "http://localhost:11434/v1".into()
}

pub fn default_response_length() -> String {
    "flexible, based on the current scene. During a conversation, keep it concise (under 150 words) to allow back-and-forth. For scene transitions, travel, or plot developments, build content (above 150 words), but allow the player to react.".to_string()
}

pub const DEFAULT_SYSTEM_PROMPT_PRESET_ID: &str = "system_default";
pub const DEFAULT_IF_SYSTEM_PROMPT_PRESET_ID: &str = "system_if_default";
pub const DEFAULT_QUANTIFIER_PROMPT_PRESET_ID: &str = "quantifier_default";
pub const DEFAULT_IMPERSONATE_PROMPT_PRESET_ID: &str = "impersonate_default";
pub const DEFAULT_OPTIONS_PROMPT_PRESET_ID: &str = "options_default";

pub fn default_active_system_prompt_preset_id() -> String {
    DEFAULT_SYSTEM_PROMPT_PRESET_ID.to_string()
}

pub fn default_active_quantifier_prompt_preset_id() -> String {
    DEFAULT_QUANTIFIER_PROMPT_PRESET_ID.to_string()
}

pub fn default_active_impersonate_prompt_preset_id() -> String {
    DEFAULT_IMPERSONATE_PROMPT_PRESET_ID.to_string()
}

pub fn default_active_options_prompt_preset_id() -> String {
    DEFAULT_OPTIONS_PROMPT_PRESET_ID.to_string()
}

/// The preset id a slot reads as when the mode registry holds no bundle for
/// `mode`. Mirrors [`default_bundle_for_mode`], plus the mode-agnostic Options
/// slot that has no bundle of its own.
pub fn default_preset_id(
    preset_type: crate::domain::model::prompt_preset::PresetType,
    mode: crate::domain::model::settings::NarratorMode,
) -> &'static str {
    use crate::domain::model::prompt_preset::PresetType;
    use crate::domain::model::settings::NarratorMode;
    match preset_type {
        PresetType::System => match mode {
            NarratorMode::Novel => DEFAULT_SYSTEM_PROMPT_PRESET_ID,
            NarratorMode::InteractiveFiction => DEFAULT_IF_SYSTEM_PROMPT_PRESET_ID,
        },
        PresetType::Quantifier => DEFAULT_QUANTIFIER_PROMPT_PRESET_ID,
        PresetType::Impersonate => DEFAULT_IMPERSONATE_PROMPT_PRESET_ID,
        PresetType::Options => DEFAULT_OPTIONS_PROMPT_PRESET_ID,
    }
}

pub fn default_allowed_modes() -> Vec<crate::domain::model::settings::NarratorMode> {
    use crate::domain::model::settings::NarratorMode;
    vec![NarratorMode::Novel, NarratorMode::InteractiveFiction]
}

pub fn default_narrator_mode() -> crate::domain::model::settings::NarratorMode {
    crate::domain::model::settings::NarratorMode::Novel
}

pub fn default_bundle_for_mode(
    mode: crate::domain::model::settings::NarratorMode,
) -> crate::domain::model::settings::ModePresetBundle {
    use crate::domain::model::prompt_preset::PresetType;
    use crate::domain::model::settings::ModePresetBundle;
    // IF mode reuses the novel quantifier/impersonate defaults;
    // only the system preset diverges (system_if_default).
    ModePresetBundle {
        mode,
        system_prompt_preset_id: default_preset_id(PresetType::System, mode).to_string(),
        quantifier_prompt_preset_id: default_preset_id(PresetType::Quantifier, mode).to_string(),
        impersonate_prompt_preset_id: default_preset_id(PresetType::Impersonate, mode).to_string(),
    }
}
