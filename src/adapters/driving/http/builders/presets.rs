//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt-preset card + form HTML builders.

use crate::adapters::driving::http::builders::forms::{textarea_field, textarea_field_readonly};
use crate::adapters::driving::http::utils::response::html_escape;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::ModePresetBundle;

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

    let novel_checked = if preset.allows_novel() {
        " checked"
    } else {
        ""
    };
    let if_checked = if preset.allows_interactive_fiction() {
        " checked"
    } else {
        ""
    };

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
        <div class="form-group">
            <label>Allowed Modes</label>
            <label class="checkbox-label"><input type="checkbox" name="allowed_mode_novel" value="true"{novel_checked} /> Novel</label>
            <label class="checkbox-label"><input type="checkbox" name="allowed_mode_if" value="true"{if_checked} /> Interactive Fiction</label>
        </div>
        <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" hx-get="/fragment/prompt-presets/{id}" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Cancel</button>
        </div>
    </form>
</div>"#,
    )
}

/// One preset card with per-mode activation: each registry bundle is
/// compared on the preset's own type slot, so an active quantifier or
/// impersonate default badges exactly like an active system default.
pub(crate) fn preset_card_html(
    preset: &PromptPreset,
    novel_bundle: &ModePresetBundle,
    if_bundle: &ModePresetBundle,
) -> String {
    let id = html_escape(&preset.id);
    let novel_active_id = preset.preset_type.bundle_slot(novel_bundle);
    let if_active_id = preset.preset_type.bundle_slot(if_bundle);
    let is_novel_active = preset.id == novel_active_id;
    let is_if_active = preset.id == if_active_id;
    let is_active = is_novel_active || is_if_active;

    let mut badges = String::new();
    if preset.is_default {
        badges.push_str(r#"<span class="badge">Default</span>"#);
    }
    if is_novel_active {
        badges.push_str(r#"<span class="badge primary">Active · Novel</span>"#);
    }
    if is_if_active {
        badges.push_str(r#"<span class="badge primary">Active · IF</span>"#);
    }

    let mut actions = String::new();
    if preset.allows_novel() && !is_novel_active {
        actions.push_str(&format!(
            r#"<button hx-post="/prompt-presets/{id}/activate?mode=novel" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active (Novel)</button>"#
        ));
    }
    if preset.allows_interactive_fiction() && !is_if_active {
        actions.push_str(&format!(
            r#"<button hx-post="/prompt-presets/{id}/activate?mode=interactive_fiction" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active (IF)</button>"#
        ));
    }
    if preset.is_default {
        actions.push_str(&format!(
            r#"<button hx-get="/fragment/prompt-presets/{id}/view" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">View</button>"#
        ));
    } else {
        actions.push_str(&format!(
            r#"<button hx-get="/fragment/prompt-presets/{id}/edit" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Edit</button>"#
        ));
        actions.push_str(&format!(
            r#"<button hx-post="/prompt-presets/{id}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>"#
        ));
    }
    actions.push_str(&format!(
        r#"<button hx-post="/prompt-presets/{id}/duplicate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-cyan">Duplicate</button>"#
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
