//! [DOC: docs/diataxis/reference/game_flow.md]
//! Worlds templates

use askama::Template;
use crate::adapters::driving::http::utils::template_helpers::{select_options_html, truncated_preview};
use crate::adapters::driving::http::view_models::{SafeHtml, SelectOptionView};
use crate::domain::model::map::MapDef;
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::world::WorldCard;

pub struct WorldRowView {
    pub key: String,
    pub name: String,
    pub description: String,
    pub game_count: usize,
}

#[derive(Template)]
#[template(
    source = r##"
<button class="btn-primary btn-new-world" hx-get="/fragment/worlds/new" hx-target="#edit-dialog-body" hx-swap="innerHTML">Create New World</button>

    {% if worlds.is_empty() %}
    <p>No worlds defined. Create your first world to get started.</p>
    {% else %}
    <ul class="worlds-list">
        {% for world in worlds %}
        <li class="world-item" id="world-{{ world.key }}">
            <div class="world-item-head">
                <strong>{{ world.name }}</strong> <em>({{ world.game_count }} {% if world.game_count == 1 %}game{% else %}games{% endif %})</em>
                <button class="btn-cyan" hx-get="/worlds/{{ world.key }}/edit" hx-target="#edit-dialog-body" hx-swap="innerHTML">Edit</button>
                <button hx-post="/worlds/{{ world.key }}/delete" hx-confirm="Delete this world? This cannot be undone." hx-swap="none" class="btn-danger">Delete</button>
            </div>
            {{ world.description }}
            <div class="inline-error-slot" data-error-slot="world-{{ world.key }}" hidden></div>
        </li>
        {% endfor %}
    </ul>
    {% endif %}
"##,
    ext = "html"
)]
pub struct WorldsPanelTemplate {
    pub worlds: Vec<WorldRowView>,
}

impl WorldsPanelTemplate {
    pub fn from_worlds(
        worlds: &[WorldCard],
        games_per_world: &std::collections::HashMap<String, usize>,
    ) -> Self {
        let rows: Vec<WorldRowView> = worlds
            .iter()
            .map(|w| {
                let game_count = games_per_world.get(&w.key).copied().unwrap_or(0);
                WorldRowView {
                    key: w.key.clone(),
                    name: w.name.clone(),
                    description: truncated_preview(&w.description),
                    game_count,
                }
            })
            .collect();
        Self { worlds: rows }
    }
}

#[derive(Template)]
#[template(
    source = r##"
{% macro posture_selects(narrator_options, perspective_options, tense_options, auto_save_key) %}<label>Narrator Mode:
                <select name="narrator_mode"{% if let Some(key) = auto_save_key %} hx-post="/worlds/{{ key }}/posture" hx-trigger="change" hx-include="closest .posture-group" hx-swap="none" data-saved-flag="world-posture-status"{% endif %}>
                    {{ narrator_options }}
                </select>
            </label>
            <label>Perspective:
                <select name="narrative_perspective"{% if let Some(key) = auto_save_key %} hx-post="/worlds/{{ key }}/posture" hx-trigger="change" hx-include="closest .posture-group" hx-swap="none" data-saved-flag="world-posture-status"{% endif %}>
                    {{ perspective_options }}
                </select>
            </label>
            <label>Tense:
                <select name="narrative_tense"{% if let Some(key) = auto_save_key %} hx-post="/worlds/{{ key }}/posture" hx-trigger="change" hx-include="closest .posture-group" hx-swap="none" data-saved-flag="world-posture-status"{% endif %}>
                    {{ tense_options }}
                </select>
            </label>{% endmacro %}
<div class="world-form-container" id="world-form-container" data-dialog-form>
    <h2 id="edit-dialog-title">{% if is_edit %}Edit World{% else %}Create New World{% endif %}</h2>

    <form hx-post="{{ form_action }}" hx-swap="none" enctype="application/x-www-form-urlencoded"{% if is_edit %} hx-on::before-request="clearPostureStatus()"{% endif %}>
        <label>Key: <input type="text" name="key" value="{{ key }}" {% if is_readonly %}readonly{% endif %} required /></label>

        <label>Name: <input type="text" name="name" value="{{ name }}" required /></label>

        <label>Description: <textarea name="description">{{ description }}</textarea></label>

        <label>Global Rules (one per line): <textarea name="global_rules">{{ global_rules }}</textarea></label>

        <label>Default Room Image: <input type="text" name="default_room_image" value="{{ default_room_image }}" /></label>

        {% if !is_edit %}
        <fieldset class="posture-group">
            <legend>Posture</legend>
            {{ posture_selects(narrator_options, perspective_options, tense_options, auto_save_key) }}
        </fieldset>
        {% endif %}

        <div class="form-group">
            <label class="checkbox-label"><input type="checkbox" name="options_always_on" value="true" {% if options_always_on %}checked{% endif %} /> Generate options after each narration</label>
        </div>

        <label>Map JSON:
            <textarea name="map_json" class="json-editor" placeholder="{{ map_placeholder }}">{{ map_json }}</textarea>
        </label>

        <label>Scenarios JSON:
            <textarea name="scenarios_json" class="json-editor" placeholder="{{ scenarios_placeholder }}">{{ scenarios_json }}</textarea>
        </label>

        <div class="form-actions">
            <button type="submit" class="btn-primary">{{ submit_text }}</button>
            <button type="button" class="btn-cyan" data-close-dialog>Cancel</button>
        </div>
        <div class="inline-error-slot" data-error-slot="world-form" hidden></div>
        {% if is_edit %}
        <p class="form-scope-note">Cancel applies to the details fields only. Posture saves automatically.</p>
        {% endif %}
    </form>

    {% if is_edit %}
    <fieldset class="posture-group" id="world-posture-group">
        <legend>Posture — saves automatically</legend>
        {{ posture_selects(narrator_options, perspective_options, tense_options, auto_save_key) }}
        <span id="world-posture-status" class="posture-status saved-flag"></span>
        <div class="inline-error-slot" data-error-slot="world-posture" hidden></div>
    </fieldset>
    {% endif %}
</div>
"##,
    ext = "html"
)]
pub struct WorldFormTemplate {
    pub is_edit: bool,
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String,
    pub default_room_image: String,
    pub map_json: String,
    pub scenarios_json: String,
    pub narrator_options: SafeHtml,
    pub perspective_options: SafeHtml,
    pub tense_options: SafeHtml,
    /// The key whose posture the selects save on change; `None` on the create form.
    pub auto_save_key: Option<String>,
    pub options_always_on: bool,
    pub form_action: String,
    pub is_readonly: bool,
    pub map_placeholder: String,
    pub scenarios_placeholder: String,
    pub submit_text: String,
}

impl WorldFormTemplate {
    pub fn from_world_data(
        world: Option<&WorldCard>,
        map: Option<&MapDef>,
        scenarios: &[StartingScenario],
    ) -> Self {
        let is_edit = world.is_some();
        let default_world = WorldCard::default();
        let w = world.unwrap_or(&default_world);

        let map_json_str = map
            .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
            .unwrap_or_default();

        let scenarios_json_str = if scenarios.is_empty() {
            String::new()
        } else {
            serde_json::to_string_pretty(scenarios).unwrap_or_default()
        };

        let (map_placeholder, scenarios_placeholder) = if is_edit {
            (String::new(), String::new())
        } else {
            (
                r##"Example: {"overworld":{"id":"overworld","name":"Overworld","regions":[]}}"##
                    .to_string(),
                "Example: []".to_string(),
            )
        };

        Self {
            is_edit,
            key: w.key.clone(),
            name: w.name.clone(),
            description: w.description.clone(),
            global_rules: w.global_rules.join("\n"),
            default_room_image: w.default_room_image.clone().unwrap_or_default(),
            map_json: map_json_str,
            scenarios_json: scenarios_json_str,
            narrator_options: select_options_html(SelectOptionView::narrator_modes(
                w.narrator_mode,
            )),
            perspective_options: select_options_html(Self::option_set(
                &[("second", "Second person"), ("third", "Third person")],
                w.narrative_perspective.as_str(),
            )),
            tense_options: select_options_html(Self::option_set(
                &[("past", "Past"), ("present", "Present")],
                w.narrative_tense.as_str(),
            )),
            auto_save_key: is_edit.then(|| w.key.clone()),
            options_always_on: w.options_always_on,
            form_action: if is_edit {
                format!("/worlds/{}", w.key)
            } else {
                "/worlds".to_string()
            },
            is_readonly: is_edit,
            map_placeholder,
            scenarios_placeholder,
            submit_text: if is_edit {
                "Update World"
            } else {
                "Create World"
            }
            .to_string(),
        }
    }

    fn option_set(values: &[(&str, &str)], selected: &str) -> Vec<SelectOptionView> {
        values
            .iter()
            .map(|(value, label)| SelectOptionView {
                value: (*value).to_string(),
                label: (*label).to_string(),
                selected: *value == selected,
            })
            .collect()
    }
}
