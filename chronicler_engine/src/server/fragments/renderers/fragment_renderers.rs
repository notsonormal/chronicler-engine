//! [DOC: docs/system/dashboard.md]
//! Fragment renderers

use askama::Template;
use crate::error::{EngineError, Result};
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, LlmMessagesTemplate, StoryLogTemplate,
    VisualSidebarTemplate,
};
use crate::server::view_models::{ActionAreaViewModel, NpcPortraitView, VisualSidebarViewModel};
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
    let ctx = state.as_game_service_context()?;
    let game_name = state
        .application_service
        .get_current_game_name(ctx)
        .unwrap_or_else(|_| "Unknown".to_string());
    render_header_unlocked(game_name)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let ctx = state.as_game_service_context()?;
    let (entries, has_last_trigger) = state
        .application_service
        .get_story_log_entries(ctx)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let entries: Vec<_> = entries.into_iter().take(MAX_LOG_DISPLAY).collect();
    let template = StoryLogTemplate::new(&entries, has_last_trigger);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let ctx = state.as_game_service_context()?;
    let (room_name, image_path) = state
        .application_service
        .get_current_room_view(ctx.clone())
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_data = state
        .application_service
        .get_npc_headshots(ctx, true)
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
    let ctx = state.as_game_service_context()?;
    let (status, phase) = state
        .application_service
        .get_input_status(ctx)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let vm = ActionAreaViewModel::new(&status, &phase);
    let template = ActionAreaTemplate::new(vm);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_character_headshots(state: &AppState) -> Result<String> {
    use crate::server::templates::CharacterHeadshotsTemplate;
    use askama::Template;

    let ctx = state.as_game_service_context()?;
    let npc_data = state
        .application_service
        .get_npc_headshots(ctx, false)
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

pub fn render_action_hints(_state: &AppState) -> Result<String> {
    Ok(String::new())
}

pub fn render_llm_messages(state: &AppState) -> Result<String> {
    let ctx = state.as_game_service_context()?;
    let messages = state
        .application_service
        .list_latest_llm_messages(ctx, 50)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let template = LlmMessagesTemplate::new(&messages);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}
