use askama::Template;

use crate::engine::logic::{get_available_exits, get_current_room};
use crate::error::Result;
use crate::model::state::GameState;
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, StoryLogTemplate, VisualSidebarTemplate,
};

const MAX_LOG_DISPLAY: usize = 50;

pub fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

fn render_header_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;
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
        .history
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
    let room = get_current_room(state)?;

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

    let npc_data: Vec<(String, String)> = if !state.scene.npcs_in_area.is_empty() {
        state
            .scene
            .npcs_in_area
            .iter()
            .filter_map(|npc| resolve_headshot(&npc.id))
            .collect()
    } else {
        // Fallback to static room.npcs
        room.npcs
            .iter()
            .filter_map(|npc_id| resolve_headshot(npc_id))
            .collect()
    };

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

    let status = state_guard.narrative.generation.status.clone();
    let phase = state_guard.narrative.generation.phase.clone();
    let exits = get_available_exits(&state_guard);
    drop(state_guard);

    let template = ActionAreaTemplate::new(&status, &phase, &exits);
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

pub fn render_action_hints(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;

    let exits = get_available_exits(&state_guard);
    let available_actions = if exits.is_empty() {
        String::from(
            "<span class=\"action-hint\">[Look]</span> <span class=\"action-hint\">[Inventory]</span>",
        )
    } else {
        let exit_hints: String = exits
            .iter()
            .map(|e| format!("<span class=\"action-hint\">[{e}]</span>"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "<span class=\"action-hint\">[Look]</span> <span class=\"action-hint\">[Inventory]</span> {exit_hints}"
        )
    };

    Ok(available_actions)
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
