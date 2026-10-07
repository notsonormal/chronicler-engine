//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt-preset card + form HTML builders.

use askama::Template;

use crate::adapters::driving::http::builders::forms::{textarea_field, textarea_field_readonly};
use crate::adapters::driving::http::prompt_presets::templates::ModeActiveIds;
use crate::adapters::driving::http::utils::handler_helpers::render_template;
use crate::adapters::driving::http::utils::response::html_escape;
use crate::adapters::driving::http::view_models::SafeHtml;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;

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

pub fn preset_edit_form_html(preset: &PromptPreset, preset_type: &str) -> String {
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

    let novel_label = NarratorMode::Novel.display_label();
    let interactive_fiction_label = NarratorMode::InteractiveFiction.display_label();

    let modes_block = if preset.preset_type == PresetType::Options {
        String::new()
    } else {
        format!(
            r#"<div class="form-group">
            <label>Allowed Modes</label>
            <label class="checkbox-label"><input type="checkbox" name="allowed_mode_novel" value="true"{novel_checked} /> {novel_label}</label>
            <label class="checkbox-label"><input type="checkbox" name="allowed_mode_if" value="true"{if_checked} /> {interactive_fiction_label}</label>
        </div>"#
        )
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
        {modes_block}
        <div class="inline-error-slot" data-error-slot="preset-edit-{id}" hidden></div>
        <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" hx-get="/fragment/prompt-presets/{id}" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Cancel</button>
        </div>
    </form>
</div>"#,
    )
}

/// The card every preset listing renders: the panel loop and the single-card
/// endpoints both go through this partial, so an edit-refreshed card is
/// identical to the same card in the panel.
///
/// `active` carries the active preset ids for the preset's own type slot, so an
/// active quantifier or impersonate default badges exactly like an active
/// system default. `preview` is pre-truncated (120 chars, newlines flattened)
/// so both renderers show the same preview.
#[derive(Template)]
#[template(
    source = r##"<div class="preset-card{% if preset.is_default %} default{% endif %}{% if is_options %}{% if is_options_active %} active{% endif %}{% else %}{% if is_novel_active || is_if_active %} active{% endif %}{% endif %}"><div class="card-header"><span class="card-title">{{ preset.name }}</span><div class="card-badges">{% if preset.is_default %}<span class="badge">Default</span>{% endif %}{% if is_options %}{% if is_options_active %}<span class="badge primary">Active</span>{% endif %}{% else %}{% if is_novel_active %}<span class="badge primary">Active · {{ self.novel_label() }}</span>{% endif %}{% if is_if_active %}<span class="badge primary">Active · {{ self.interactive_fiction_label() }}</span>{% endif %}{% endif %}</div></div><div class="card-details preset-preview">{{ preview }}</div><div class="inline-error-slot" data-error-slot="preset-{{ preset.id }}" hidden></div><div class="card-actions">{% if is_options %}{% if !is_options_active %}<button hx-post="/prompt-presets/{{ preset.id }}/activate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active</button>{% endif %}{% else %}{% if preset.allows_novel() && !is_novel_active %}<button hx-post="/prompt-presets/{{ preset.id }}/activate?mode=novel" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active ({{ self.novel_label() }})</button>{% endif %}{% if preset.allows_interactive_fiction() && !is_if_active %}<button hx-post="/prompt-presets/{{ preset.id }}/activate?mode=interactive_fiction" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">Set Active ({{ self.interactive_fiction_label() }})</button>{% endif %}{% endif %}{% if preset.is_default %}<button hx-get="/fragment/prompt-presets/{{ preset.id }}/view" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">View</button>{% else %}<button hx-get="/fragment/prompt-presets/{{ preset.id }}/edit" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Edit</button><button hx-post="/prompt-presets/{{ preset.id }}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>{% endif %}<button hx-post="/prompt-presets/{{ preset.id }}/duplicate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-cyan">Duplicate</button></div></div>"##,
    ext = "html"
)]
pub struct PresetCardTemplate {
    pub preset: PromptPreset,
    pub preview: String,
    pub is_novel_active: bool,
    pub is_if_active: bool,
    pub is_options: bool,
    pub is_options_active: bool,
}

impl PresetCardTemplate {
    pub fn new(preset: &PromptPreset, active: &ModeActiveIds) -> Self {
        let is_novel_active = preset.id == active.novel;
        let is_if_active = preset.id == active.interactive_fiction;
        Self {
            preset: preset.clone(),
            preview: truncated_preview(preset),
            is_novel_active,
            is_if_active,
            is_options: preset.preset_type == PresetType::Options,
            is_options_active: false,
        }
    }

    pub fn new_options(preset: &PromptPreset, active_options_id: &str) -> Self {
        Self {
            is_options_active: preset.id == active_options_id,
            ..Self::new(preset, &ModeActiveIds::default())
        }
    }

    pub fn novel_label(&self) -> &'static str {
        NarratorMode::Novel.display_label()
    }

    pub fn interactive_fiction_label(&self) -> &'static str {
        NarratorMode::InteractiveFiction.display_label()
    }
}

fn truncated_preview(preset: &PromptPreset) -> String {
    preset
        .preview_text()
        .chars()
        .take(120)
        .collect::<String>()
        .replace('\n', " ")
}

pub(crate) fn preset_card_html(preset: &PromptPreset, active: &ModeActiveIds) -> String {
    render_template(PresetCardTemplate::new(preset, active)).0
}

pub(crate) fn options_preset_card_html(preset: &PromptPreset, active_options_id: &str) -> String {
    render_template(PresetCardTemplate::new_options(preset, active_options_id)).0
}

pub(crate) fn preset_card_view(preset: &PromptPreset, active: &ModeActiveIds) -> SafeHtml {
    SafeHtml::new(preset_card_html(preset, active))
}

pub(crate) fn options_card_view(preset: &PromptPreset, active_options_id: &str) -> SafeHtml {
    SafeHtml::new(options_preset_card_html(preset, active_options_id))
}
