//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games templates

use askama::Template;
use crate::adapters::driving::http::utils::template_helpers::select_options_html;
use crate::adapters::driving::http::view_models::{SafeHtml, SelectOptionView};
use crate::domain::model::game::Game;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::world::WorldCard;

pub struct GameRowView {
    pub id: u64,
    pub display_name: String,
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
                <span class="game-name">{{ game.display_name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <span class="persona-badge">{{ game.persona_name }}</span>
            </div>
            <div class="game-actions">
                <details class="game-rename">
                    <summary>Rename</summary>
                    <form hx-post="/games/{{ game.id }}/rename" hx-swap="none">
                        <input type="text" name="display_name" value="{{ game.display_name }}" required maxlength="120" aria-label="Game display name">
                        <button type="submit" class="btn-primary">Save</button>
                    </form>
                </details>
                <button class="btn-reset-small" hx-post="/reset" hx-confirm="Reset the current game? All progress will be lost." hx-swap="none" title="Reset game">&#x21bb;</button>
            </div>
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
                <label for="new-game-world">World</label>
                <select id="new-game-world" name="world_key" required>
                    {% for world in worlds %}
                    <option value="{{ world.key }}" title="{{ world.description }}">{{ world.name }}</option>
                    {% endfor %}
                </select>
            </div>
            <div class="form-row">
                {% if personas.is_empty() %}
                <div class="games-empty">No personas available. Create a persona first.</div>
                {% else %}
                <label for="new-game-persona">Persona</label>
                <select id="new-game-persona" name="persona_key" required>
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
            <div class="games-empty">No other saved games.</div>
            {% else %}
            {% for game in saved_games %}
            <div class="game-item" data-id="{{ game.id }}">
                <span class="game-name">{{ game.display_name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <span class="persona-badge">{{ game.persona_name }}</span>
                <div class="game-actions">
                    <details class="game-rename">
                        <summary>Rename</summary>
                        <form hx-post="/games/{{ game.id }}/rename" hx-swap="none">
                            <input type="text" name="display_name" value="{{ game.display_name }}" required maxlength="120" aria-label="Game display name">
                            <button type="submit" class="btn-primary">Save</button>
                        </form>
                    </details>
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

/// In-game posture override (mode/perspective/tense) + per-game preset
/// picker for the active game. Every dropdown auto-saves on change and
/// re-renders this fragment.
#[derive(Template)]
#[template(
    source = r##"
<div class="posture-override" id="game-posture-controls">
    <div class="posture-row">
        <label>Mode
            <select name="narrator_mode" id="game-narrator-mode" hx-post="/games/{{ game_id }}/mode" hx-trigger="change" hx-target="#game-posture-controls" hx-swap="outerHTML">
                {{ mode_options }}
            </select>
        </label>
        <label>Perspective
            <select name="narrative_perspective" id="game-narrative-perspective" hx-post="/games/{{ game_id }}/posture" hx-trigger="change" hx-include="closest .posture-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                <option value="second"{% if perspective == "second" %} selected{% endif %}>Second person</option>
                <option value="third"{% if perspective == "third" %} selected{% endif %}>Third person</option>
            </select>
        </label>
        <label>Tense
            <select name="narrative_tense" id="game-narrative-tense" hx-post="/games/{{ game_id }}/posture" hx-trigger="change" hx-include="closest .posture-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
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
            <select name="system_preset_id" id="game-system-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                {% for opt in system_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Quantifier
            <select name="quantifier_preset_id" id="game-quantifier-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
                {% for opt in quantifier_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Impersonate
            <select name="impersonate_preset_id" id="game-impersonate-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-target="#game-posture-controls" hx-swap="outerHTML">
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
    pub mode_options: SafeHtml,
    pub perspective: String,
    pub tense: String,
    pub system_options: Vec<SelectOptionView>,
    pub quantifier_options: Vec<SelectOptionView>,
    pub impersonate_options: Vec<SelectOptionView>,
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
            mode_options: select_options_html(SelectOptionView::narrator_modes(game.narrator_mode)),
            perspective: game.narrative_perspective.as_str().to_string(),
            tense: game.narrative_tense.as_str().to_string(),
            system_options: SelectOptionView::presets(
                system_presets,
                game.narrator_mode,
                &game.active_system_prompt_preset_id,
            ),
            quantifier_options: SelectOptionView::presets(
                quantifier_presets,
                game.narrator_mode,
                &game.active_quantifier_prompt_preset_id,
            ),
            impersonate_options: SelectOptionView::presets(
                impersonate_presets,
                game.narrator_mode,
                &game.active_impersonate_prompt_preset_id,
            ),
            preset_load_error: None,
        }
    }
}
