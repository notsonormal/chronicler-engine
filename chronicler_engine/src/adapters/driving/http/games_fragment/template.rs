//! [DOC: docs/system/dashboard.md]
//! Games templates

use askama::Template;
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
    source = r#"
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
"#,
    ext = "html"
)]
pub struct GamesPanelTemplate {
    pub active_game: Option<GameRowView>,
    pub saved_games: Vec<GameRowView>,
    pub worlds: Vec<WorldCard>,
    pub personas: Vec<PersonaRowView>,
}
