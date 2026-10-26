//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The parts of the screen: the one list of part names, each with its renderer.

use crate::adapters::driving::http::builders::headers::header_fragment_html;
use crate::adapters::driving::http::games::handlers::render_games_panel;
use crate::adapters::driving::http::layout::handlers::render_status_display;
use crate::adapters::driving::http::prompt_presets::handlers::render_prompt_presets_panel;
use crate::adapters::driving::http::settings::handlers::render_settings_panel;
use crate::adapters::driving::http::worlds::handlers::render_worlds_panel;
use crate::adapters::driving::http::AppState;
use crate::domain::model::game::GameId;
use crate::error::Result;

pub type PartRenderer = fn(&AppState, Option<GameId>) -> Result<String>;

/// Every part of the screen, in the order of the shell, each with its renderer. A
/// part has one name and one renderer, so a name with no renderer cannot exist.
pub const SCREEN_PARTS: [(&str, PartRenderer); 11] = [
    ("header", render_header_part as PartRenderer),
    ("failure-banner", render_failure_banner_part as PartRenderer),
    ("story-log", render_story_log_part as PartRenderer),
    ("visual-sidebar", render_visual_sidebar_part as PartRenderer),
    ("options-dock", render_options_dock_part as PartRenderer),
    ("status-display", render_status_display_part as PartRenderer),
    ("settings", render_settings_part as PartRenderer),
    ("prompt-presets", render_prompt_presets_part as PartRenderer),
    ("worlds", render_worlds_part as PartRenderer),
    ("games", render_games_part as PartRenderer),
    ("llm-messages", render_llm_messages_part as PartRenderer),
];

/// The part names in the order of `SCREEN_PARTS`. Only the guardrail and the
/// tests that compare another list of parts with the server's call it.
pub fn screen_part_names() -> impl Iterator<Item = &'static str> {
    SCREEN_PARTS.iter().map(|(name, _)| *name)
}

const NO_ACTIVE_GAME: &str = "No active game";

fn render_header_part(state: &AppState, game: Option<GameId>) -> Result<String> {
    match game {
        Some(game_id) => state.render_header(game_id),
        None => header_fragment_html(NO_ACTIVE_GAME.to_string()),
    }
}

fn render_failure_banner_part(state: &AppState, _game: Option<GameId>) -> Result<String> {
    state.render_failure_banner()
}

/// The Game-tab parts that show one Game's data render empty with no Game.
fn render_story_log_part(state: &AppState, game: Option<GameId>) -> Result<String> {
    game.map_or_else(
        || Ok(String::new()),
        |game_id| state.render_story_log(game_id),
    )
}

fn render_visual_sidebar_part(state: &AppState, game: Option<GameId>) -> Result<String> {
    game.map_or_else(
        || Ok(String::new()),
        |game_id| state.render_visual_sidebar(game_id),
    )
}

fn render_options_dock_part(state: &AppState, game: Option<GameId>) -> Result<String> {
    game.map_or_else(
        || Ok(String::new()),
        |game_id| state.render_options_dock(game_id),
    )
}

fn render_status_display_part(state: &AppState, game: Option<GameId>) -> Result<String> {
    Ok(render_status_display(state, game))
}

fn render_settings_part(state: &AppState, _game: Option<GameId>) -> Result<String> {
    render_settings_panel(state)
}

fn render_prompt_presets_part(state: &AppState, _game: Option<GameId>) -> Result<String> {
    render_prompt_presets_panel(state)
}

fn render_worlds_part(state: &AppState, _game: Option<GameId>) -> Result<String> {
    render_worlds_panel(state)
}

fn render_games_part(state: &AppState, game: Option<GameId>) -> Result<String> {
    render_games_panel(state, game)
}

fn render_llm_messages_part(state: &AppState, _game: Option<GameId>) -> Result<String> {
    state.render_llm_messages()
}
