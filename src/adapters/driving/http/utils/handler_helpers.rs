//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Handler-level utilities: shared template render + option string + preset helpers.

use axum::response::Html;
use uuid::Uuid;

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

/// Preset ids are storage keys: a bare wall-clock id collides when two presets
/// are created in the same millisecond, and the loser is silently overwritten
/// (create → duplicate is the exact flow the UI offers). The uuid-v4 suffix
/// makes same-millisecond creations distinct (residual ~2⁻³² per pair).
pub(crate) fn generate_preset_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(
        "preset-{millis}-{}",
        &Uuid::new_v4().simple().to_string()[..8]
    )
}
