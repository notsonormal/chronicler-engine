use askama::Template;

use crate::error::{EngineError, Result};
use crate::model::state::GameState;
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, LlmMessagesTemplate, StoryLogTemplate,
    VisualSidebarTemplate,
};

const MAX_LOG_DISPLAY: usize = 50;

pub fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

fn render_header_unlocked(state: &GameState) -> Result<String> {
    let room = state
        .current_room()
        .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;
    let template = HeaderTemplate {
        room_name: room.name.clone(),
    };
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_header(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;
    render_header_unlocked(&state_guard)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;

    let entries: Vec<_> = state_guard
        .narrative
        .history()
        .iter()
        .take(MAX_LOG_DISPLAY)
        .cloned()
        .collect();
    let template = StoryLogTemplate::new(&entries);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

fn render_visual_sidebar_unlocked(state: &GameState) -> Result<String> {
    let room = state
        .current_room()
        .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

    let image_path = room
        .image_path
        .clone()
        .or_else(|| state.world.default_room_image.clone());

    let resolve_headshot = |npc_id: &str| {
        let npc = state.npcs.get(npc_id)?;
        let image_path = npc.sheet.preferred_image()?.to_string();
        let name = npc.sheet.name.clone();
        Some((image_path, name))
    };

    let npc_data: Vec<(String, String)> = state
        .scene
        .npcs_in_area
        .iter()
        .filter_map(|npc| resolve_headshot(&npc.id))
        .collect();

    let template = VisualSidebarTemplate::new(image_path, room.name.clone(), npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;
    render_visual_sidebar_unlocked(&state_guard)
}

/// [DOC: docs/system/game_flow.md]
pub fn render_action_area(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;

    let status = state_guard.narrative.input_buffer.status.clone();
    let phase = state_guard.narrative.input_buffer.phase.clone();
    drop(state_guard);

    let template = ActionAreaTemplate::new(&status, &phase, &[]);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

/// [DOC: docs/system/game_flow.md]
pub fn render_character_headshots(state: &AppState) -> Result<String> {
    use crate::server::templates::CharacterHeadshotsTemplate;
    use askama::Template;

    let state_guard = state.load_state()?;

    let npc_data: Vec<(String, String)> = state_guard
        .npcs
        .iter()
        .filter_map(|(_npc_id, npc)| {
            let image = npc.sheet.preferred_image()?;
            let name = npc.sheet.name.clone();
            Some((image.to_string(), name))
        })
        .collect();

    let template = CharacterHeadshotsTemplate::new(npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_action_hints(_state: &AppState) -> Result<String> {
    Ok(String::new())
}

pub fn render_llm_messages(state: &AppState) -> Result<String> {
    let messages = state.llm_message_storage.list_latest(50).map_err(|e| {
        crate::error::EngineError::Template(format!("Failed to load LLM messages: {e}"))
    })?;
    let template = LlmMessagesTemplate::new(&messages);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
