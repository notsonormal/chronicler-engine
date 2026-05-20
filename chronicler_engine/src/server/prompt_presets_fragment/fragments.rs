use crate::model::prompt_preset::PromptPreset;
use crate::server::fragments::html_escape;

pub(crate) fn preset_edit_form_html(
    preset: &PromptPreset,
    preset_type: &str,
    _is_active: bool,
) -> String {
    format!(
        r#"<div class="preset-card edit-form">
    <div class="card-header">
        <span class="card-title">Edit {}</span>
    </div>
    <form hx-post="/prompt-presets/{}" hx-target="closest .preset-card" hx-swap="outerHTML">
        <input type="hidden" name="preset_type" value="{}" />
        <div class="form-group">
            <label for="edit-name-{}">Name</label>
            <input type="text" id="edit-name-{}" name="name" value="{}" required />
        </div>
        <div class="form-group">
            <label for="edit-text-{}">Prompt Text</label>
            <textarea id="edit-text-{}" name="prompt_text" rows="10" required>{}</textarea>
        </div>
        <div class="form-actions">
            <button type="submit" class="primary">Save</button>
            <button type="button" hx-get="/fragment/prompt-presets/{}" hx-target="closest .preset-card" hx-swap="outerHTML">Cancel</button>
        </div>
    </form>
</div>"#,
        html_escape(&preset.name),
        html_escape(&preset.id),
        html_escape(preset_type),
        html_escape(&preset.id),
        html_escape(&preset.id),
        html_escape(&preset.name),
        html_escape(&preset.id),
        html_escape(&preset.id),
        html_escape(&preset.prompt_text),
        html_escape(&preset.id),
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
            r#"<button hx-post="/prompt-presets/{}/activate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="primary">Set Active</button>"#,
            html_escape(&preset.id)
        ));
    }
    if !preset.is_default {
        actions.push_str(&format!(
            r#"<button hx-get="/fragment/prompt-presets/{}/edit" hx-target="closest .preset-card" hx-swap="outerHTML">Edit</button>"#,
            html_escape(&preset.id)
        ));
        actions.push_str(&format!(
            r#"<button hx-post="/prompt-presets/{}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="danger">Delete</button>"#,
            html_escape(&preset.id)
        ));
    }

    let preview: String = preset
        .prompt_text
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
