//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The parts of the screen: the one list of part names, each with its renderer.

use crate::adapters::driving::http::builders::headers::header_fragment_html;
use crate::adapters::driving::http::games::handlers::render_games_panel;
use crate::adapters::driving::http::layout::handlers::{
    render_autoplay_controls, render_send_control, render_status_display,
};
use crate::adapters::driving::http::prompt_presets::handlers::render_prompt_presets_panel;
use crate::adapters::driving::http::settings::handlers::render_settings_panel;
use crate::adapters::driving::http::worlds::handlers::render_worlds_panel;
use crate::adapters::driving::http::AppState;
use crate::domain::model::game::GameId;
use crate::error::Result;

/// What a screen load shows: the active Game, and the page of the saved Games that
/// the Games part lists.
pub struct ScreenView {
    pub game: Option<GameId>,
    pub games_page: u64,
}

pub type PartRenderer = fn(&AppState, &ScreenView) -> Result<String>;

/// Every part of the screen, in the order of the shell, each with its renderer. A
/// part has one name and one renderer, so a name with no renderer cannot exist.
pub const SCREEN_PARTS: [(&str, PartRenderer); 13] = [
    ("header", render_header_part as PartRenderer),
    ("failure-banner", render_failure_banner_part as PartRenderer),
    ("story-log", render_story_log_part as PartRenderer),
    ("visual-sidebar", render_visual_sidebar_part as PartRenderer),
    ("options-dock", render_options_dock_part as PartRenderer),
    ("send-control", render_send_control_part as PartRenderer),
    (
        "autoplay-controls",
        render_autoplay_controls_part as PartRenderer,
    ),
    ("status-display", render_status_display_part as PartRenderer),
    ("settings", render_settings_part as PartRenderer),
    ("prompt-presets", render_prompt_presets_part as PartRenderer),
    ("worlds", render_worlds_part as PartRenderer),
    ("games", render_games_part as PartRenderer),
    ("llm-messages", render_llm_messages_part as PartRenderer),
];

pub fn screen_part_names() -> impl Iterator<Item = &'static str> {
    SCREEN_PARTS.iter().map(|(name, _)| *name)
}

const NO_ACTIVE_GAME: &str = "No active game";

fn render_header_part(state: &AppState, view: &ScreenView) -> Result<String> {
    match view.game {
        Some(game_id) => state.render_header(game_id),
        None => header_fragment_html(NO_ACTIVE_GAME.to_string()),
    }
}

fn render_failure_banner_part(state: &AppState, _view: &ScreenView) -> Result<String> {
    state.render_failure_banner()
}

/// The Game-tab parts that show one Game's data render empty with no Game.
fn render_story_log_part(state: &AppState, view: &ScreenView) -> Result<String> {
    view.game.map_or_else(
        || Ok(String::new()),
        |game_id| state.render_story_log(game_id),
    )
}

fn render_visual_sidebar_part(state: &AppState, view: &ScreenView) -> Result<String> {
    view.game.map_or_else(
        || Ok(String::new()),
        |game_id| state.render_visual_sidebar(game_id),
    )
}

fn render_options_dock_part(state: &AppState, view: &ScreenView) -> Result<String> {
    view.game.map_or_else(
        || Ok(String::new()),
        |game_id| state.render_options_dock(game_id),
    )
}

fn render_send_control_part(state: &AppState, view: &ScreenView) -> Result<String> {
    Ok(render_send_control(state, view.game))
}

fn render_autoplay_controls_part(state: &AppState, view: &ScreenView) -> Result<String> {
    Ok(render_autoplay_controls(state, view.game))
}

fn render_status_display_part(state: &AppState, view: &ScreenView) -> Result<String> {
    Ok(render_status_display(state, view.game))
}

fn render_settings_part(state: &AppState, _view: &ScreenView) -> Result<String> {
    render_settings_panel(state)
}

fn render_prompt_presets_part(state: &AppState, _view: &ScreenView) -> Result<String> {
    render_prompt_presets_panel(state)
}

fn render_worlds_part(state: &AppState, _view: &ScreenView) -> Result<String> {
    render_worlds_panel(state)
}

fn render_games_part(state: &AppState, view: &ScreenView) -> Result<String> {
    render_games_panel(state, view.game, view.games_page)
}

fn render_llm_messages_part(state: &AppState, _view: &ScreenView) -> Result<String> {
    state.render_llm_messages()
}
