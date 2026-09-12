//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games templates

use askama::Template;
use crate::domain::model::game::Game;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::world::WorldCard;

pub struct GameRowView {
    pub id: u64,
    pub name: String,
    pub world_name: String,
    pub persona_name: String,
}

pub struct PersonaRowView {
    pub key: String,
    pub name: String,
}

#[derive(Template)]
#[template(
    source = r##"
<div class="games-panel">
    <div class="games-section">
        <h2>Active Game</h2>
        {% match active_game %}
        {% when Some(game) %}
        <div class="game-item active">
            <div class="active-game-info">
                <span class="game-name">{{ game.name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <span class="persona-badge">{{ game.persona_name }}</span>
                <span class="game-badge">Current</span>
            </div>
            <button class="btn-reset-small" hx-post="/reset" hx-confirm="Reset the current game? All progress will be lost." hx-swap="none" title="Reset game">&#x21bb;</button>
        </div>
        {{ posture_html|safe }}
        {% when None %}
        <div class="game-item"><span class="game-name">No active game</span></div>
        {% endmatch %}
    </div>

    <div class="games-section new-game-section">
        <h2>New Game</h2>
        {% if worlds.is_empty() %}
        <div class="games-empty">No worlds available. Create a world first.</div>
        {% else %}
        <form class="new-game-form" hx-post="/games" hx-swap="none">
            <div class="form-row">
                <select name="world_key" required>
                    {% for world in worlds %}
                    <option value="{{ world.key }}" title="{{ world.description }}">{{ world.name }}</option>
                    {% endfor %}
                </select>
            </div>
            <div class="form-row">
                {% if personas.is_empty() %}
                <div class="games-empty">No personas available. Create a persona first.</div>
                {% else %}
                <select name="persona_key" required>
                    {% for p in personas %}
                    <option value="{{ p.key }}">{{ p.name }}</option>
                    {% endfor %}
                </select>
                {% endif %}
            </div>
            <div class="form-row">
                <button type="submit" class="btn-primary"
                    {% if personas.is_empty() %}disabled{% endif %}>Start New Game</button>
            </div>
        </form>
        {% endif %}
    </div>

    <div class="games-section">
        <h2>Saved Games</h2>
        <div class="games-list">
            {% if saved_games.is_empty() %}
            <div class="games-empty">No saved games.</div>
            {% else %}
            {% for game in saved_games %}
            <div class="game-item" data-id="{{ game.id }}">
                <span class="game-name">{{ game.name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <span class="persona-badge">{{ game.persona_name }}</span>
                <div class="game-actions">
                    <button class="btn-primary" hx-post="/games/{{ game.id }}/switch" hx-swap="none">Switch</button>
                    <button class="btn-danger" hx-post="/games/{{ game.id }}/delete" hx-target="closest .game-item" hx-swap="outerHTML" hx-confirm="Delete this game? This cannot be undone.">Delete</button>
                </div>
            </div>
            {% endfor %}
            {% endif %}
        </div>
    </div>
</div>
"##,
    ext = "html"
)]
pub struct GamesPanelTemplate {
    pub active_game: Option<GameRowView>,
    pub saved_games: Vec<GameRowView>,
    pub worlds: Vec<WorldCard>,
    pub personas: Vec<PersonaRowView>,
    /// Rendered `GamePostureTemplate`; empty when no game is active.
    /// Pre-rendered because the fragment is also a standalone auto-save target.
    pub posture_html: String,
}

/// One `<option>` in a per-game preset picker dropdown.
pub struct PresetOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

/// In-game posture override (mode/perspective/tense) + per-game preset
/// picker for the active game. Every dropdown auto-saves on change and
/// re-renders this fragment.
#[derive(Template)]
#[template(
    source = r##"
<div class="posture-override" id="game-posture-controls">
    <div class="posture-row">
        <label>Mode
            <select name="narrator_mode" hx-post="/games/{{ game_id }}/mode" hx-trigger="change" hx-target="#game-posture-controls" hx-swap="outerHTML">
                <option value="novel"{% if mode == "novel" %} selected{% endif %}>Novel</option>
                <option value="interactive_fiction"{% if mode == "interactive_fiction" %} selected{% endif %}>Interactive Fiction</option>
            </select>
        </label>
        <label>Perspective
            <select name="narrative_perspective" hx-post="/games/{{ game_id }}/posture" hx-trigger="change" hx-include="closest .posture-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                <option value="second"{% if perspective == "second" %} selected{% endif %}>Second person</option>
                <option value="third"{% if perspective == "third" %} selected{% endif %}>Third person</option>
            </select>
        </label>
        <label>Tense
            <select name="narrative_tense" hx-post="/games/{{ game_id }}/posture" hx-trigger="change" hx-include="closest .posture-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                <option value="past"{% if tense == "past" %} selected{% endif %}>Past</option>
                <option value="present"{% if tense == "present" %} selected{% endif %}>Present</option>
            </select>
        </label>
    </div>
    <div class="preset-picker-row">
        {% match preset_load_error %}
        {% when Some with (err) %}
        <div class="error-message">Presets unavailable: {{ err }}</div>
        {% when None %}
        <label>System
            <select name="system_preset_id" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                {% for opt in system_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Quantifier
            <select name="quantifier_preset_id" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                {% for opt in quantifier_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Impersonate
            <select name="impersonate_preset_id" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                {% for opt in impersonate_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        {% endmatch %}
    </div>
</div>
"##,
    ext = "html"
)]
pub struct GamePostureTemplate {
    pub game_id: u64,
    pub mode: String,
    pub perspective: String,
    pub tense: String,
    pub system_options: Vec<PresetOptionView>,
    pub quantifier_options: Vec<PresetOptionView>,
    pub impersonate_options: Vec<PresetOptionView>,
    /// Set when the preset library failed to load: the picker row renders
    /// this message instead of the three selects.
    pub preset_load_error: Option<String>,
}

impl GamePostureTemplate {
    pub fn from_game(
        game: &Game,
        system_presets: &[PromptPreset],
        quantifier_presets: &[PromptPreset],
        impersonate_presets: &[PromptPreset],
    ) -> Self {
        Self {
            game_id: game.id,
            mode: game.narrator_mode.as_str().to_string(),
            perspective: game.narrative_perspective.as_str().to_string(),
            tense: game.narrative_tense.as_str().to_string(),
            system_options: PresetOptionView::options(
                system_presets,
                game.narrator_mode,
                &game.active_system_prompt_preset_id,
            ),
            quantifier_options: PresetOptionView::options(
                quantifier_presets,
                game.narrator_mode,
                &game.active_quantifier_prompt_preset_id,
            ),
            impersonate_options: PresetOptionView::options(
                impersonate_presets,
                game.narrator_mode,
                &game.active_impersonate_prompt_preset_id,
            ),
            preset_load_error: None,
        }
    }
}

impl PresetOptionView {
    /// Picker options for one slot: presets allowing the game's mode, plus
    /// the stored selection (even when disallowed or absent from the
    /// library) so the browser never silently substitutes another preset.
    pub fn options(
        presets: &[PromptPreset],
        mode: NarratorMode,
        selected_id: &str,
    ) -> Vec<PresetOptionView> {
        let mut options: Vec<PresetOptionView> = presets
            .iter()
            .filter(|p| p.allows(mode) || p.id == selected_id)
            .map(|p| PresetOptionView {
                value: p.id.clone(),
                label: p.name.clone(),
                selected: p.id == selected_id,
            })
            .collect();
        if !options.iter().any(|o| o.value == selected_id) {
            options.insert(
                0,
                PresetOptionView {
                    value: selected_id.to_string(),
                    label: format!("(missing) {selected_id}"),
                    selected: true,
                },
            );
        }
        options
    }
}
