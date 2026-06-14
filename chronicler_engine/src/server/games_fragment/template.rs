//! [DOC: docs/system/dashboard.md]
//! Games templates

use askama::Template;
use crate::model::world::WorldCard;

pub struct GameRowView {
    pub id: u64,
    pub name: String,
    pub world_name: String,
}

#[derive(Template)]
#[template(
    source = r#"
<div class="save-load-panel">
    <div class="save-load-section">
        <h2>Active Game</h2>
        {% match active_game %}
        {% when Some(game) %}
        <div class="game-item active">
            <span class="game-name">{{ game.name }}</span>
            <span class="world-badge">{{ game.world_name }}</span>
            <span class="game-badge">Current</span>
        </div>
        {% when None %}
        <div class="game-item"><span class="game-name">No active game</span></div>
        {% endmatch %}
    </div>

    <div class="save-load-section">
        <h2>Saved Games</h2>
        <div class="games-list">
            {% if saved_games.is_empty() %}
            <div class="games-empty">No saved games.</div>
            {% else %}
            {% for game in saved_games %}
            <div class="game-item" data-id="{{ game.id }}">
                <span class="game-name">{{ game.name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <div class="game-actions">
                    <button class="btn-switch" hx-post="/games/{{ game.id }}/switch" hx-swap="none">Switch</button>
                    <button class="btn-delete" hx-post="/games/{{ game.id }}/delete" hx-target="closest .game-item" hx-swap="outerHTML" hx-confirm="Delete this game? This cannot be undone.">Delete</button>
                </div>
            </div>
            {% endfor %}
            {% endif %}
        </div>
    </div>

    <div class="save-load-actions">
        {% if !worlds.is_empty() %}
        <details class="world-picker">
            <summary>New Game</summary>
            <form hx-post="/games" hx-swap="none">
                <select name="world_key" required>
                    {% for world in worlds %}
                    <option value="{{ world.key }}" title="{{ world.description }}">{{ world.name }}</option>
                    {% endfor %}
                </select>
                <button type="submit" class="btn-primary">Create Game</button>
            </form>
        </details>
        {% endif %}
        <button class="btn-reset" hx-post="/reset" hx-confirm="Are you sure you want to reset the current game? All progress will be lost." hx-swap="none">Reset Current Game</button>
    </div>
</div>
"#,
    ext = "html"
)]
pub struct GamesPanelTemplate {
    pub active_game: Option<GameRowView>,
    pub saved_games: Vec<GameRowView>,
    pub worlds: Vec<WorldCard>,
}
