//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Form field HTML builders.

use crate::adapters::driving::http::utils::response::html_escape;
use crate::domain::model::settings::NarratorMode;

/// The `<option>` pair for a narrator-mode select, marking `selected_mode`.
/// Shared by the world form and the game posture controls so the two
/// selectors cannot drift.
pub(crate) fn narrator_mode_select_options_html(selected_mode: &str) -> String {
    [NarratorMode::Novel, NarratorMode::InteractiveFiction]
        .into_iter()
        .map(|mode| {
            let selected = if mode.as_str() == selected_mode {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{}"{}>{}</option>"#,
                mode.as_str(),
                selected,
                mode.display_label()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn textarea_field(
    id: &str,
    label: &str,
    name: &str,
    value: Option<&str>,
    rows: usize,
) -> String {
    let value = value.unwrap_or("");
    format!(
        r#"<div class="form-group">
    <label for="{id}">{label}</label>
    <textarea id="{id}" name="{name}" rows="{rows}">{value}</textarea>
</div>"#,
        id = html_escape(id),
        label = html_escape(label),
        name = html_escape(name),
        rows = rows,
        value = html_escape(value),
    )
}

pub(crate) fn textarea_field_readonly(label: &str, value: Option<&str>, rows: usize) -> String {
    let value = value.unwrap_or("");
    format!(
        r#"<div class="form-group">
    <label>{label}</label>
    <textarea rows="{rows}" disabled>{value}</textarea>
</div>"#,
        label = html_escape(label),
        rows = rows,
        value = html_escape(value),
    )
}
