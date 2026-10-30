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
    pub run_held: bool,
}

/// The pager of the saved Games. A list of one page has none.
pub struct GamesPagerView {
    pub page: u64,
    pub page_count: u64,
    pub previous_page: u64,
    pub next_page: u64,
    pub previous_disabled: bool,
    pub next_disabled: bool,
}

impl GamesPagerView {
    pub fn new(page: u64, page_count: u64) -> Option<Self> {
        (page_count > 1).then(|| Self {
            page,
            page_count,
            previous_page: page.saturating_sub(1).max(1),
            next_page: (page + 1).min(page_count),
            previous_disabled: page <= 1,
            next_disabled: page >= page_count,
        })
    }
}

pub struct PersonaRowView {
    pub key: String,
    pub name: String,
}

#[derive(Template)]
#[template(
    source = r##"
{% macro run_badge(run_held) %}{% if run_held %}<span class="run-badge" title="An Autoplay run holds this game"><svg class="icon icon-spin" aria-hidden="true"><use href="#i-loader-circle"/></svg> Autoplay</span>{% endif %}{% endmacro %}
{% macro rename_button(id) %}<button type="button" class="btn-cyan" id="game-rename-open-{{ id }}" hx-get="/fragment/games/{{ id }}/rename" hx-target="#edit-dialog-body" hx-swap="innerHTML">Rename</button>{% endmacro %}
<div class="games-section">
    <h2>Active Game</h2>
    {% match active_game %}
    {% when Some(game) %}
    <div class="game-item active" id="game-row-{{ game.id }}">
        <div class="active-game-info">
            <span class="game-name">{{ game.display_name }}</span>
            <span class="world-badge">{{ game.world_name }}</span>
            <span class="persona-badge">{{ game.persona_name }}</span>{{ run_badge(game.run_held) }}
        </div>
        <div class="game-actions">
            {{ rename_button(game.id) }}
            <button class="btn-reset-small" hx-post="/reset" hx-confirm="Reset the current game? All progress will be lost." hx-swap="none" title="Reset game" aria-label="Reset game"><svg class="icon" aria-hidden="true"><use href="#i-rotate-ccw"/></svg></button>
        </div>
    </div>
    {{ posture_html|safe }}
    {% when None %}
    <div class="game-item"><span class="game-name">No active game</span></div>
    {% endmatch %}
</div>

<div class="games-section new-game-section">
    {% if worlds.is_empty() %}
    <h2>New Game</h2>
    <div class="games-empty">No worlds available. Create a world first.</div>
    {% else %}
    <button type="button" class="btn-cyan" id="game-new-open" hx-get="/fragment/games/new" hx-target="#edit-dialog-body" hx-swap="innerHTML">New Game</button>
    {% endif %}
</div>

<div class="games-section">
    <h2>Saved Games</h2>
    <div class="games-list">
        {% if saved_games.is_empty() %}
        <div class="games-empty">No other saved games.</div>
        {% else %}
        {% for game in saved_games %}
        <div class="game-item" id="game-row-{{ game.id }}" data-id="{{ game.id }}">
            <span class="game-name">{{ game.display_name }}</span>
            <span class="world-badge">{{ game.world_name }}</span>
            <span class="persona-badge">{{ game.persona_name }}</span>{{ run_badge(game.run_held) }}
            <div class="game-actions">
                {{ rename_button(game.id) }}
                <button class="btn-primary" hx-post="/games/{{ game.id }}/switch" hx-swap="none">Switch</button>
                <button class="btn-danger" hx-post="/games/{{ game.id }}/delete" hx-swap="none" hx-confirm="Delete this game? This cannot be undone.">Delete</button>
            </div>
        </div>
        {% endfor %}
        {% endif %}
    </div>
    {% if let Some(pager) = pager %}
    <nav class="games-pager" aria-label="Saved Games pages">
        <button type="button" class="btn-cyan games-page-previous" data-page="{{ pager.previous_page }}"{% if pager.previous_disabled %} aria-disabled="true"{% endif %}>Previous</button>
        <span class="games-page-position">Page {{ pager.page }} of {{ pager.page_count }}</span>
        <button type="button" class="btn-cyan games-page-next" data-page="{{ pager.next_page }}"{% if pager.next_disabled %} aria-disabled="true"{% endif %}>Next</button>
    </nav>
    {% endif %}
</div>
"##,
    ext = "html"
)]
pub struct GamesPanelTemplate {
    pub active_game: Option<GameRowView>,
    pub saved_games: Vec<GameRowView>,
    pub pager: Option<GamesPagerView>,
    pub worlds: Vec<WorldCard>,
    /// Rendered before the panel, because the preset load behind it can fail on its own.
    pub posture_html: String,
}

#[derive(Template)]
#[template(
    source = r##"
<div class="game-rename-form" data-dialog-form>
    <h2 id="edit-dialog-title">Rename game</h2>
    <form hx-post="/games/{{ id }}/rename" hx-swap="none">
        <input type="text" name="display_name" value="{{ display_name }}" required maxlength="120" aria-label="Game display name">
        <div class="inline-error-slot" data-error-slot="game-rename" hidden></div>
        <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" class="btn-cyan" data-close-dialog>Cancel</button>
        </div>
    </form>
</div>
"##,
    ext = "html"
)]
pub struct GameRenameFormTemplate {
    pub id: u64,
    pub display_name: String,
}

#[derive(Template)]
#[template(
    source = r##"
<div class="new-game-form-dialog" data-dialog-form>
    <h2 id="edit-dialog-title">New Game</h2>
    {% if worlds.is_empty() %}
    <div class="games-empty">No worlds available. Create a world first.</div>
    <div class="form-actions">
        <button type="button" class="btn-cyan" data-close-dialog>Close</button>
    </div>
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
        <div class="inline-error-slot" data-error-slot="new-game" hidden></div>
        <div class="form-actions">
            <button type="submit" class="btn-primary"{% if personas.is_empty() %} disabled{% endif %}>Start New Game</button>
            <button type="button" class="btn-cyan" data-close-dialog>Cancel</button>
        </div>
    </form>
    {% endif %}
</div>
"##,
    ext = "html"
)]
pub struct NewGameFormTemplate {
    pub worlds: Vec<WorldCard>,
    pub personas: Vec<PersonaRowView>,
}

#[derive(Template)]
#[template(
    source = r##"
{% macro disabled(on) %}{% if on %} disabled{% endif %}{% endmacro %}
<div class="posture-override" id="game-posture-controls">
    <div class="posture-row">
        <label>Mode
            <select name="narrator_mode" id="game-narrator-mode" hx-post="/games/{{ game_id }}/mode" hx-trigger="change" hx-swap="none"{{ disabled(view_only) }}>
                {{ mode_options }}
            </select>
        </label>
        <label>Perspective
            <select name="narrative_perspective" id="game-narrative-perspective" hx-post="/games/{{ game_id }}/posture" hx-trigger="change" hx-include="closest .posture-row" hx-swap="none"{{ disabled(view_only) }}>
                <option value="second"{% if perspective == "second" %} selected{% endif %}>Second person</option>
                <option value="third"{% if perspective == "third" %} selected{% endif %}>Third person</option>
            </select>
        </label>
        <label>Tense
            <select name="narrative_tense" id="game-narrative-tense" hx-post="/games/{{ game_id }}/posture" hx-trigger="change" hx-include="closest .posture-row" hx-swap="none"{{ disabled(view_only) }}>
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
            <select name="system_preset_id" id="game-system-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-swap="none"{{ disabled(view_only) }}>
                {% for opt in system_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Quantifier
            <select name="quantifier_preset_id" id="game-quantifier-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-swap="none"{{ disabled(view_only) }}>
                {% for opt in quantifier_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Impersonate
            <select name="impersonate_preset_id" id="game-impersonate-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-swap="none"{{ disabled(view_only) }}>
                {% for opt in impersonate_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
            </select>
        </label>
        <label>Options
            <select name="options_preset_id" id="game-options-preset" hx-post="/games/{{ game_id }}/presets" hx-trigger="change" hx-include="closest .preset-picker-row" hx-swap="none"{{ disabled(view_only) }}>
                {% for opt in options_options %}<option value="{{ opt.value }}"{% if opt.selected %} selected{% endif %}>{{ opt.label }}</option>{% endfor %}
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
    pub options_options: Vec<SelectOptionView>,
    pub preset_load_error: Option<String>,
    pub view_only: bool,
}

impl GamePostureTemplate {
    pub fn from_game(
        game: &Game,
        system_presets: &[PromptPreset],
        quantifier_presets: &[PromptPreset],
        impersonate_presets: &[PromptPreset],
        options_presets: &[PromptPreset],
        view_only: bool,
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
            options_options: SelectOptionView::every_preset(
                options_presets,
                &game.active_options_prompt_preset_id,
            ),
            preset_load_error: None,
            view_only,
        }
    }
}
