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

pub fn default_active_system_prompt_preset_id() -> String {
    "system_default".to_string()
}

pub fn default_active_quantifier_prompt_preset_id() -> String {
    "quantifier_default".to_string()
}

pub fn default_active_impersonate_prompt_preset_id() -> String {
    "impersonate_default".to_string()
}

pub fn default_if_system_prompt_preset_id() -> String {
    "system_if_default".to_string()
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
    use crate::domain::model::settings::{ModePresetBundle, NarratorMode};
    // IF mode reuses the novel quantifier/impersonate defaults;
    // only the system preset diverges (system_if_default).
    let system_prompt_preset_id = match mode {
        NarratorMode::Novel => default_active_system_prompt_preset_id(),
        NarratorMode::InteractiveFiction => default_if_system_prompt_preset_id(),
    };
    ModePresetBundle {
        mode,
        system_prompt_preset_id,
        quantifier_prompt_preset_id: default_active_quantifier_prompt_preset_id(),
        impersonate_prompt_preset_id: default_active_impersonate_prompt_preset_id(),
    }
}

pub fn default_narrative_perspective() -> crate::domain::model::settings::NarrativePerspective {
    crate::domain::model::settings::NarrativePerspective::Third
}

pub fn default_narrative_tense() -> crate::domain::model::settings::NarrativeTense {
    crate::domain::model::settings::NarrativeTense::Past
}
