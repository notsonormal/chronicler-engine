//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Prompt-preset card + form HTML builders.

use crate::adapters::driving::http::builders::forms::{textarea_field, textarea_field_readonly};
use crate::adapters::driving::http::fragments::html_escape;
use crate::domain::model::prompt_preset::PromptPreset;

pub(crate) fn preset_view_form_html(preset: &PromptPreset) -> String {
    let id = html_escape(&preset.id);
    let name = html_escape(&preset.name);

    let role_field = textarea_field_readonly("Role", preset.role.as_deref(), 4);
    let instructions_field =
        textarea_field_readonly("Instructions", preset.instructions.as_deref(), 10);
    let writing_style_field =
        textarea_field_readonly("Writing Style", preset.writing_style.as_deref(), 4);
    let output_format_field =
        textarea_field_readonly("Output Format", preset.output_format.as_deref(), 6);

    format!(
        r#"<div class="preset-card view-form">
    <div class="card-header">
        <span class="card-title">View {name}</span>
    </div>
    <div class="form-group">
        <label>Name</label>
        <input type="text" value="{name}" readonly />
    </div>
    {role_field}
    {instructions_field}
    {writing_style_field}
    {output_format_field}
    <div class="form-actions">
        <button type="button" hx-get="/fragment/prompt-presets/{id}" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Close</button>
    </div>
</div>"#,
    )
}

pub(crate) fn preset_edit_form_html(
    preset: &PromptPreset,
    preset_type: &str,
    _is_active: bool,
) -> String {
    let id = html_escape(&preset.id);
    let name = html_escape(&preset.name);
    let preset_type_escaped = html_escape(preset_type);

    let role_field = textarea_field(
        &format!("edit-role-{}", preset.id),
        "Role",
        "role",
        preset.role.as_deref(),
        4,
    );
    let instructions_field = textarea_field(
        &format!("edit-instructions-{}", preset.id),
        "Instructions",
        "instructions",
        preset.instructions.as_deref(),
        10,
    );
    let writing_style_field = textarea_field(
        &format!("edit-style-{}", preset.id),
        "Writing Style",
        "writing_style",
        preset.writing_style.as_deref(),
        4,
    );
    let output_format_field = textarea_field(
        &format!("edit-output-{}", preset.id),
        "Output Format",
        "output_format",
        preset.output_format.as_deref(),
        6,
    );

    format!(
        r#"<div class="preset-card edit-form">
    <div class="card-header">
        <span class="card-title">Edit {name}</span>
    </div>
    <form hx-post="/prompt-presets/{id}" hx-target="closest .preset-card" hx-swap="outerHTML">
        <input type="hidden" name="preset_type" value="{preset_type_escaped}" />
        <div class="form-group">
            <label for="edit-name-{id}">Name</label>
            <input type="text" id="edit-name-{id}" name="name" value="{name}" required />
        </div>
        {role_field}
        {instructions_field}
        {writing_style_field}
        {output_format_field}
        <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" hx-get="/fragment/prompt-presets/{id}" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Cancel</button>
        </div>
    </form>
</div>"#,
    )
}

pub(crate) fn preset_card_html(preset: &PromptPreset, is_active: bool) -> String {
    let mut badges = String::new();
    if preset.is_default {
        badges.push_str(r#"<span class="badge">Default</span>"#);
    }
    if is_active {
        badges.push_str(r#" <span class="badge primary">Active</span>"#);
    }

    let mut actions = String::new();
    if !is_active {
        actions.push_str(&format!(
            r#"<button hx-post="/prompt-presets/{}/activate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active</button>"#,
            html_escape(&preset.id)
        ));
    }
    if preset.is_default {
        actions.push_str(&format!(
            r#"<button hx-get="/fragment/prompt-presets/{}/view" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">View</button>"#,
            html_escape(&preset.id)
        ));
    } else {
        actions.push_str(&format!(
            r#"<button hx-get="/fragment/prompt-presets/{}/edit" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Edit</button>"#,
            html_escape(&preset.id)
        ));
        actions.push_str(&format!(
            r#"<button hx-post="/prompt-presets/{}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>"#,
            html_escape(&preset.id)
        ));
    }
    actions.push_str(&format!(
        r#"<button hx-post="/prompt-presets/{}/duplicate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-cyan">Duplicate</button>"#,
        html_escape(&preset.id)
    ));

    let preview: String = preset
        .preview_text()
        .chars()
        .take(120)
        .collect::<String>()
        .replace('\n', " ");

    format!(
        r#"<div class="preset-card{}{}">
    <div class="card-header">
        <span class="card-title">{}</span>
        <div class="card-badges">{}</div>
    </div>
    <div class="card-details preset-preview">{}</div>
    <div class="card-actions">{}</div>
</div>"#,
        if preset.is_default { " default" } else { "" },
        if is_active { " active" } else { "" },
        html_escape(&preset.name),
        badges,
        html_escape(&preview),
        actions,
    )
}
