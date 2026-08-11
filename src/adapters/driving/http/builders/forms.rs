//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Textarea field HTML builders.

use crate::adapters::driving::http::utils::response::html_escape;

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
