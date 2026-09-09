//! [DOC: docs/diataxis/reference/game_flow.md]
//! Worlds templates

use askama::Template;
use crate::domain::model::world::WorldCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::scenario::StartingScenario;

pub struct WorldRowView {
    pub key: String,
    pub name: String,
    pub description: String,
    pub game_count: usize,
}

#[derive(Template)]
#[template(
    source = r##"
<div class="worlds-panel">
    <button class="btn-primary btn-new-world" hx-get="/fragment/worlds/new" hx-target=".worlds-panel" hx-swap="outerHTML">Create New World</button>

    {% if worlds.is_empty() %}
    <p>No worlds defined. Create your first world to get started.</p>
    {% else %}
    <ul class="worlds-list">
        {% for world in worlds %}
        <li class="world-item">
            <strong>{{ world.name }}</strong> - {{ world.description }} <em>({{ world.game_count }} games)</em>
            <button class="btn-cyan" hx-get="/worlds/{{ world.key }}/edit" hx-target=".worlds-panel" hx-swap="outerHTML">Edit</button>
            <button hx-post="/worlds/{{ world.key }}/delete" hx-confirm="Delete this world? This cannot be undone." hx-target="closest .world-item" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>
        </li>
        {% endfor %}
    </ul>
    {% endif %}
</div>
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
                    description: w.description.clone(),
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
<div class="worlds-panel">
<div class="world-form-container">
    <h2>{% if is_edit %}Edit World{% else %}Create New World{% endif %}</h2>

    <form hx-post="{{ form_action }}" hx-target=".worlds-panel" hx-swap="outerHTML" enctype="application/x-www-form-urlencoded">
        <label>Key: <input type="text" name="key" value="{{ key }}" {% if is_readonly %}readonly{% endif %} required /></label>

        <label>Name: <input type="text" name="name" value="{{ name }}" required /></label>

        <label>Description: <textarea name="description">{{ description }}</textarea></label>

        <label>Global Rules (one per line): <textarea name="global_rules">{{ global_rules }}</textarea></label>

        <label>Default Room Image: <input type="text" name="default_room_image" value="{{ default_room_image }}" /></label>

        <div class="form-group posture-group">
            <label>Narrator Mode:
                <select name="narrator_mode" {% if is_edit %}hx-post="/worlds/{{ key }}/posture" hx-trigger="change" hx-include="closest .posture-group" hx-target="#world-posture-status" hx-swap="innerHTML"{% endif %}>
                    <option value="novel" {% if narrator_mode == "novel" %}selected{% endif %}>Novel</option>
                    <option value="interactive_fiction" {% if narrator_mode == "interactive_fiction" %}selected{% endif %}>Interactive Fiction</option>
                </select>
            </label>
            <label>Perspective:
                <select name="narrative_perspective" {% if is_edit %}hx-post="/worlds/{{ key }}/posture" hx-trigger="change" hx-include="closest .posture-group" hx-target="#world-posture-status" hx-swap="innerHTML"{% endif %}>
                    <option value="second" {% if narrative_perspective == "second" %}selected{% endif %}>Second person</option>
                    <option value="third" {% if narrative_perspective == "third" %}selected{% endif %}>Third person</option>
                </select>
            </label>
            <label>Tense:
                <select name="narrative_tense" {% if is_edit %}hx-post="/worlds/{{ key }}/posture" hx-trigger="change" hx-include="closest .posture-group" hx-target="#world-posture-status" hx-swap="innerHTML"{% endif %}>
                    <option value="past" {% if narrative_tense == "past" %}selected{% endif %}>Past</option>
                    <option value="present" {% if narrative_tense == "present" %}selected{% endif %}>Present</option>
                </select>
            </label>
            <span id="world-posture-status"></span>
        </div>

        <label>Map JSON:
            <textarea name="map_json" class="json-editor" placeholder="{{ map_placeholder }}">{{ map_json }}</textarea>
        </label>

        <label>Scenarios JSON:
            <textarea name="scenarios_json" class="json-editor" placeholder="{{ scenarios_placeholder }}">{{ scenarios_json }}</textarea>
        </label>

        <div class="form-actions">
            <button type="submit" class="btn-primary">{{ submit_text }}</button>
            <button type="button" class="btn-cyan" hx-get="/fragment/worlds" hx-target=".worlds-panel" hx-swap="outerHTML">Cancel</button>
        </div>
    </form>
</div>
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
    pub narrator_mode: String,
    pub narrative_perspective: String,
    pub narrative_tense: String,
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
                r##"{"overworld":{"id":"overworld","name":"Overworld","regions":[]}}"##.to_string(),
                "[]".to_string(),
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
            narrator_mode: w.narrator_mode.as_str().to_string(),
            narrative_perspective: w.narrative_perspective.as_str().to_string(),
            narrative_tense: w.narrative_tense.as_str().to_string(),
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
}
