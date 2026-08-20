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
