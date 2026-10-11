//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt-preset card + form HTML builders.

use askama::Template;

use crate::adapters::driving::http::builders::forms::{textarea_field, textarea_field_readonly};
use crate::adapters::driving::http::utils::handler_helpers::render_template;
use crate::adapters::driving::http::utils::response::html_escape;
use crate::adapters::driving::http::view_models::SafeHtml;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{AppSettings, NarratorMode};

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

/// The card every preset listing renders, so an edit-refreshed card matches the panel's.
/// `preview` arrives pre-truncated: newlines flattened, a cut marked with an ellipsis.
#[derive(Template)]
#[template(
    source = r##"<div class="preset-card{% if preset.is_default %} default{% endif %}{% if !self.active_badges.is_empty() %} active{% endif %}"><div class="card-header"><span class="card-title">{{ preset.name }}</span><div class="card-badges">{% if preset.is_default %}<span class="badge">Default</span>{% endif %}{% for badge in self.active_badges %}<span class="badge primary">{{ badge }}</span>{% endfor %}</div></div><div class="card-details preset-preview">{{ preview }}</div><div class="inline-error-slot" data-error-slot="preset-{{ preset.id }}" hidden></div><div class="card-actions">{% for (label, mode_query) in self.activate_buttons %}<button hx-post="/prompt-presets/{{ preset.id }}/activate{{ mode_query }}" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-primary">{{ label }}</button>{% endfor %}{% if preset.is_default %}<button hx-get="/fragment/prompt-presets/{{ preset.id }}/view" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">View</button>{% else %}<button hx-get="/fragment/prompt-presets/{{ preset.id }}/edit" hx-target="closest .preset-card" hx-swap="outerHTML" class="btn-cyan">Edit</button><button hx-post="/prompt-presets/{{ preset.id }}/delete" hx-confirm="Delete this preset?" hx-target="closest .preset-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>{% endif %}<button hx-post="/prompt-presets/{{ preset.id }}/duplicate" hx-target=".prompt-presets-panel" hx-swap="outerHTML" class="btn-cyan">Duplicate</button></div></div>"##,
    ext = "html"
)]
pub struct PresetCardTemplate {
    pub preset: PromptPreset,
    pub preview: String,
    pub active_badges: Vec<String>,
    /// Each entry pairs the button's label with the mode query appended to its
    /// activate route (empty for the mode-agnostic Options slot).
    pub activate_buttons: Vec<(String, String)>,
}

impl PresetCardTemplate {
    pub fn from_settings(preset: &PromptPreset, settings: &AppSettings) -> Self {
        let (active_badges, activate_buttons) = if preset.preset_type.is_mode_tagged() {
            mode_slot_actions(preset, settings)
        } else {
            options_slot_actions(preset, settings)
        };
        Self {
            preset: preset.clone(),
            preview: truncated_preview(preset),
            active_badges,
            activate_buttons,
        }
    }
}

fn mode_slot_actions(
    preset: &PromptPreset,
    settings: &AppSettings,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut active_badges = Vec::new();
    let mut activate_buttons = Vec::new();
    for mode in [NarratorMode::Novel, NarratorMode::InteractiveFiction] {
        let is_active = settings.active_preset_id(preset.preset_type, mode) == preset.id;
        if is_active {
            active_badges.push(format!("Active · {}", mode.display_label()));
        } else if preset.allows(mode) {
            activate_buttons.push((
                format!("Set Active ({})", mode.display_label()),
                format!("?mode={}", mode.as_str()),
            ));
        }
    }
    (active_badges, activate_buttons)
}

/// The Options slot is settings-level, so its `allowed_modes` flags do not gate the
/// button.
fn options_slot_actions(
    preset: &PromptPreset,
    settings: &AppSettings,
) -> (Vec<String>, Vec<(String, String)>) {
    let is_active = settings.active_preset_id(preset.preset_type, NarratorMode::Novel) == preset.id;
    if is_active {
        (vec!["Active".to_string()], Vec::new())
    } else {
        (Vec::new(), vec![("Set Active".to_string(), String::new())])
    }
}

/// The ellipsis stops a cut preview reading as the whole field.
fn truncated_preview(preset: &PromptPreset) -> String {
    const MAX_PREVIEW_CHARS: usize = 120;
    let text = preset.preview_text().replace('\n', " ");
    let text = text.trim();
    if text.chars().count() <= MAX_PREVIEW_CHARS {
        return text.to_string();
    }
    let head: String = text.chars().take(MAX_PREVIEW_CHARS).collect();
    match head.rsplit_once(' ') {
        Some((words, _)) => format!("{}…", words.trim_end()),
        None => format!("{head}…"),
    }
}

pub(crate) fn preset_card_html(preset: &PromptPreset, settings: &AppSettings) -> String {
    render_template(PresetCardTemplate::from_settings(preset, settings)).0
}

pub(crate) fn preset_card_view(preset: &PromptPreset, settings: &AppSettings) -> SafeHtml {
    SafeHtml::new(preset_card_html(preset, settings))
}
