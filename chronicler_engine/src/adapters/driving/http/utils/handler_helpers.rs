//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Handler-level utilities: shared template render + option string + preset helpers.

use axum::response::Html;

use crate::domain::model::prompt_preset::PresetType;

/// Render an `askama::Template` to `Html<String>`, falling back to an error span on failure.
pub(crate) fn render_template<T: askama::Template>(template: T) -> Html<String> {
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
    }
}

/// Empty string → `None`; otherwise `Some(value.to_string())`.
pub(crate) fn opt_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub(crate) fn parse_preset_type(value: &str) -> Option<PresetType> {
    PresetType::try_from(value).ok()
}

pub(crate) fn generate_preset_id() -> String {
    format!(
        "preset-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}
