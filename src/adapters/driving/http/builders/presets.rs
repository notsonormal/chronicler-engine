//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt-preset card + form HTML builders.

use askama::Template;

use crate::adapters::driving::http::builders::forms::{textarea_field, textarea_field_readonly};
use crate::adapters::driving::http::utils::fragment::render_or_error_fragment;
use crate::adapters::driving::http::utils::template_helpers::truncated_preview;
use crate::adapters::driving::http::view_models::SafeHtml;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{AppSettings, NarratorMode};

#[derive(Template)]
#[template(
    source = r##"<div class="preset-form view-form" data-dialog-form>
    <h2 id="edit-dialog-title">View {{ name }}</h2>
    <div class="form-group">
        <label>Name</label>
        <input type="text" value="{{ name }}" readonly />
    </div>
    {{ role_field }}
    {{ instructions_field }}
    {{ writing_style_field }}
    {{ output_format_field }}
    <div class="form-actions">
        <button type="button" class="btn-cyan" data-close-dialog>Close</button>
    </div>
</div>"##,
    ext = "html"
)]
struct PresetViewFormTemplate {
    name: String,
    role_field: SafeHtml,
    instructions_field: SafeHtml,
    writing_style_field: SafeHtml,
    output_format_field: SafeHtml,
}

pub fn preset_view_form_html(preset: &PromptPreset) -> String {
    render_or_error_fragment(&PresetViewFormTemplate {
        name: preset.name.clone(),
        role_field: SafeHtml::new(textarea_field_readonly("Role", preset.role.as_deref(), 4)),
        instructions_field: SafeHtml::new(textarea_field_readonly(
            "Instructions",
            preset.instructions.as_deref(),
            10,
        )),
        writing_style_field: SafeHtml::new(textarea_field_readonly(
            "Writing Style",
            preset.writing_style.as_deref(),
            4,
        )),
        output_format_field: SafeHtml::new(textarea_field_readonly(
            "Output Format",
            preset.output_format.as_deref(),
            6,
        )),
    })
}

/// An Options preset serves both modes, so it has no Allowed Modes.
#[derive(Template)]
#[template(
    source = r##"<div class="preset-form edit-form" data-dialog-form>
    <h2 id="edit-dialog-title">Edit {{ name }}</h2>
    <form hx-post="/prompt-presets/{{ id }}" hx-swap="none" data-refusal-what="the preset was not saved">
        <input type="hidden" name="preset_type" value="{{ preset_type }}" />
        <div class="form-group">
            <label for="edit-name-{{ id }}">Name</label>
            <input type="text" id="edit-name-{{ id }}" name="name" value="{{ name }}" required />
        </div>
        {{ role_field }}
        {{ instructions_field }}
        {{ writing_style_field }}
        {{ output_format_field }}
        {% if shows_allowed_modes %}
        <div class="form-group">
            <label>Allowed Modes</label>
            <label class="checkbox-label"><input type="checkbox" name="allowed_mode_novel" value="true"{% if novel_checked %} checked{% endif %} /> {{ novel_label }}</label>
            <label class="checkbox-label"><input type="checkbox" name="allowed_mode_if" value="true"{% if interactive_fiction_checked %} checked{% endif %} /> {{ interactive_fiction_label }}</label>
        </div>
        {% endif %}
        <div class="inline-error-slot" data-error-slot="preset-edit-{{ id }}" hidden></div>
        <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" class="btn-cyan" data-close-dialog>Cancel</button>
        </div>
    </form>
</div>"##,
    ext = "html"
)]
struct PresetEditFormTemplate {
    id: String,
    name: String,
    preset_type: String,
    role_field: SafeHtml,
    instructions_field: SafeHtml,
    writing_style_field: SafeHtml,
    output_format_field: SafeHtml,
    shows_allowed_modes: bool,
    novel_checked: bool,
    interactive_fiction_checked: bool,
    novel_label: &'static str,
    interactive_fiction_label: &'static str,
}

pub fn preset_edit_form_html(preset: &PromptPreset, preset_type: &str) -> String {
    let field = |label_id: &str, label: &str, name: &str, value: &Option<String>, rows: usize| {
        SafeHtml::new(textarea_field(
            &format!("{label_id}-{}", preset.id),
            label,
            name,
            value.as_deref(),
            rows,
        ))
    };
    render_or_error_fragment(&PresetEditFormTemplate {
        id: preset.id.clone(),
        name: preset.name.clone(),
        preset_type: preset_type.to_string(),
        role_field: field("edit-role", "Role", "role", &preset.role, 4),
        instructions_field: field(
            "edit-instructions",
            "Instructions",
            "instructions",
            &preset.instructions,
            10,
        ),
        writing_style_field: field(
            "edit-style",
            "Writing Style",
            "writing_style",
            &preset.writing_style,
            4,
        ),
        output_format_field: field(
            "edit-output",
            "Output Format",
            "output_format",
            &preset.output_format,
            6,
        ),
        shows_allowed_modes: preset.preset_type != PresetType::Options,
        novel_checked: preset.allows_novel(),
        interactive_fiction_checked: preset.allows_interactive_fiction(),
        novel_label: NarratorMode::Novel.display_label(),
        interactive_fiction_label: NarratorMode::InteractiveFiction.display_label(),
    })
}

/// `preview` arrives pre-truncated: newlines flattened, a cut marked with an ellipsis.
#[derive(Template)]
#[template(
    source = r##"<div class="preset-card{% if preset.is_default %} default{% endif %}{% if !self.active_badges.is_empty() %} active{% endif %}" id="preset-card-{{ preset.id }}"><div class="card-header"><span class="card-title">{{ preset.name }}</span><div class="card-badges">{% if preset.is_default %}<span class="badge">Default</span>{% endif %}{% for badge in self.active_badges %}<span class="badge primary">{{ badge }}</span>{% endfor %}</div></div><div class="card-details preset-preview">{{ preview }}</div><div class="inline-error-slot" data-error-slot="preset-{{ preset.id }}" hidden></div><div class="card-actions">{% for button in self.activate_buttons %}<button hx-post="/prompt-presets/{{ preset.id }}/activate{{ button.mode_query }}" hx-swap="none" class="btn-primary">{{ button.label }}</button>{% endfor %}{% if preset.is_default %}<button hx-get="/fragment/prompt-presets/{{ preset.id }}/view" hx-target="#edit-dialog-body" hx-swap="innerHTML" class="btn-cyan">View</button>{% else %}<button hx-get="/fragment/prompt-presets/{{ preset.id }}/edit" hx-target="#edit-dialog-body" hx-swap="innerHTML" class="btn-cyan">Edit</button><button hx-post="/prompt-presets/{{ preset.id }}/delete" hx-confirm="Delete this preset?" hx-swap="none" class="btn-danger">Delete</button>{% endif %}<button hx-post="/prompt-presets/{{ preset.id }}/duplicate" hx-swap="none" class="btn-cyan">Duplicate</button></div></div>"##,
    ext = "html"
)]
pub struct PresetCardTemplate {
    pub preset: PromptPreset,
    pub preview: String,
    pub active_badges: Vec<String>,
    pub activate_buttons: Vec<ActivateButton>,
}

/// `mode_query` is `?mode=<narrator mode>` for a mode-tagged preset, empty for
/// the Options slot's button.
pub struct ActivateButton {
    pub label: String,
    pub mode_query: String,
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
            preview: truncated_preview(preset.preview_text()),
            active_badges,
            activate_buttons,
        }
    }
}

fn mode_slot_actions(
    preset: &PromptPreset,
    settings: &AppSettings,
) -> (Vec<String>, Vec<ActivateButton>) {
    let mut active_badges = Vec::new();
    let mut activate_buttons = Vec::new();
    for mode in [NarratorMode::Novel, NarratorMode::InteractiveFiction] {
        let is_active = settings.active_preset_id(preset.preset_type, mode) == preset.id;
        if is_active {
            active_badges.push(format!("Active · {}", mode.display_label()));
        } else if preset.allows(mode) {
            activate_buttons.push(ActivateButton {
                label: format!("Set Active ({})", mode.display_label()),
                mode_query: format!("?mode={}", mode.as_str()),
            });
        }
    }
    (active_badges, activate_buttons)
}

/// The Options slot is settings-level, so the Novel slot decides its button.
fn options_slot_actions(
    preset: &PromptPreset,
    settings: &AppSettings,
) -> (Vec<String>, Vec<ActivateButton>) {
    let is_active = settings.active_preset_id(preset.preset_type, NarratorMode::Novel) == preset.id;
    if is_active {
        (vec!["Active".to_string()], Vec::new())
    } else {
        (
            Vec::new(),
            vec![ActivateButton {
                label: "Set Active".to_string(),
                mode_query: String::new(),
            }],
        )
    }
}

pub(crate) fn preset_card_html(preset: &PromptPreset, settings: &AppSettings) -> String {
    render_or_error_fragment(&PresetCardTemplate::from_settings(preset, settings))
}

pub(crate) fn preset_card_view(preset: &PromptPreset, settings: &AppSettings) -> SafeHtml {
    SafeHtml::new(preset_card_html(preset, settings))
}
