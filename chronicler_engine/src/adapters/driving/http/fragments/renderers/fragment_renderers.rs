//! [DOC: docs/system/dashboard.md]
//! Fragment renderers

use askama::Template;
use crate::application::query_handlers;
use crate::error::{EngineError, Result};
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::templates::{
    ActionAreaTemplate, HeaderTemplate, LlmMessagesTemplate, StoryLogTemplate,
    VisualSidebarTemplate,
};
use crate::adapters::driving::http::view_models::{
    ActionAreaViewModel, NpcPortraitView, VisualSidebarViewModel,
};
use super::response::html_escape;

const MAX_LOG_DISPLAY: usize = 50;

pub fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

fn render_header_unlocked(game_name: String) -> Result<String> {
    let template = HeaderTemplate { game_name };
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_header(state: &AppState) -> Result<String> {
    let game_name = query_handlers::get_current_game_name(&state.application_service)
        .unwrap_or_else(|_| "Unknown".to_string());
    render_header_unlocked(game_name)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let (entries, has_last_trigger) =
        query_handlers::get_story_log_entries(&state.application_service)
            .map_err(|e| EngineError::Config(e.to_string()))?;

    let entries: Vec<_> = entries.into_iter().take(MAX_LOG_DISPLAY).collect();
    let template = StoryLogTemplate::new(&entries, has_last_trigger);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let (room_name, image_path) = query_handlers::get_current_room_view(&state.application_service)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_data = query_handlers::get_npc_headshots(&state.application_service, true)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_portraits: Vec<NpcPortraitView> = npc_data
        .into_iter()
        .map(|(image_path, name)| NpcPortraitView { image_path, name })
        .collect();

    let vm = VisualSidebarViewModel::new(image_path, room_name, npc_portraits);
    let template = VisualSidebarTemplate::new(vm);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_action_area(state: &AppState) -> Result<String> {
    let (status, phase) = query_handlers::get_input_status(&state.application_service)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let vm = ActionAreaViewModel::new(&status, &phase);
    let template = ActionAreaTemplate::new(vm);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_character_headshots(state: &AppState) -> Result<String> {
    use crate::adapters::driving::http::templates::CharacterHeadshotsTemplate;
    use askama::Template;

    let npc_data = query_handlers::get_npc_headshots(&state.application_service, false)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_portraits: Vec<NpcPortraitView> = npc_data
        .into_iter()
        .map(|(image_path, name)| NpcPortraitView { image_path, name })
        .collect();

    let template = CharacterHeadshotsTemplate::new(npc_portraits);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_llm_messages(state: &AppState) -> Result<String> {
    let messages = query_handlers::list_latest_llm_messages(&state.application_service, 50)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let template = LlmMessagesTemplate::new(&messages);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}
