//! Pure template builder functions for testable UI fragments.
//!
//! These functions build Askama templates from GameState without requiring AppState or locks.
//! They return Template structs (not rendered strings) to allow:
//! - Unit testing without HTTP context
//! - Composable fragment building
//! - Clear separation between data transformation and rendering
//!
//! Usage:
//!     let template = build_header_template(state)?;
//!     let html = template.render()?;
//!
//! The builders mirror the rendering functions in fragments.rs but operate
//! on pure &GameState references, making them fully testable.

use crate::engine::logic::{get_available_exits, get_current_room};
use crate::error::Result;
use crate::model::state::GameState;

use super::templates::{
    ActionAreaTemplate, HeaderTemplate, NpcDetailsTemplate, StoryLogTemplate, VisualSidebarTemplate,
};

/// Maximum number of log entries to display in the story log.
const MAX_LOG_DISPLAY: usize = 50;

// =============================================================================
// HTML Escaping Utilities
// =============================================================================

/// Escape HTML special characters in a string.
///
/// This is a simple utility for escaping user-provided content that won't go
/// through Askama's automatic escaping (e.g., for error messages).
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render an error message for the UI.
///
/// Returns an HTML error div.
pub fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

// =============================================================================
// Header Template Builder
// =============================================================================

/// Build a HeaderTemplate from game state.
pub fn build_header_template(_state: &GameState) -> Result<HeaderTemplate> {
    Ok(HeaderTemplate {})
}

// =============================================================================
// Story Log Template Builder
// =============================================================================

/// Build a StoryLogTemplate from game state.
pub fn build_story_log_template(state: &GameState) -> Result<StoryLogTemplate> {
    let entries: Vec<_> = state
        .narration_history
        .iter()
        .take(MAX_LOG_DISPLAY)
        .cloned()
        .collect();
    Ok(StoryLogTemplate::new(&entries))
}

// =============================================================================
// Visual Sidebar Template Builder
// =============================================================================

/// Build a VisualSidebarTemplate from game state.
pub fn build_visual_sidebar_template(state: &GameState) -> Result<VisualSidebarTemplate> {
    let room = get_current_room(state)?;

    // Collect exits as direction strings
    let exits: Vec<String> = room
        .exits
        .keys()
        .map(|dir| format!("{dir:?}").to_lowercase())
        .collect();

    // Collect NPC data: (image_path, name, id) tuples
    let npc_data: Vec<(String, String, String)> = room
        .npcs
        .iter()
        .filter_map(|npc_id| {
            let npc = state.npcs.get(npc_id)?;
            let image_path = npc.sheet.image_path.as_ref()?.clone();
            let name = npc.sheet.name.clone();
            // Include the NPC ID
            Some((image_path, name, npc_id.clone()))
        })
        .collect();

    Ok(VisualSidebarTemplate::new(
        room.image_path.clone(),
        room.name.clone(),
        exits,
        npc_data,
    ))
}

// =============================================================================
// Action Area Template Builder
// =============================================================================

/// Build an ActionAreaTemplate from game state.
pub fn build_action_area_template(state: &GameState) -> Result<ActionAreaTemplate> {
    let is_generating = state.tui_state.is_generating;
    let error_message = state.tui_state.error_message.clone();
    let exits = get_available_exits(state);
    Ok(ActionAreaTemplate::new_with_error(is_generating, &exits, error_message))
}

// =============================================================================
// NPC Details Template Builder
// =============================================================================

/// Build an NpcDetailsTemplate from an NPC ID and game state.
pub fn build_npc_details_template(state: &GameState, npc_id: &str) -> Result<NpcDetailsTemplate> {
    let npc = state
        .npcs
        .get(npc_id)
        .ok_or_else(|| crate::error::EngineError::NpcNotFound(npc_id.to_string()))?;

    Ok(NpcDetailsTemplate::new(
        npc.sheet.name.clone(),
        npc.sheet.description.clone(),
        npc.sheet.personality.clone(),
        npc.sheet.scenario.clone(),
        npc.sheet.image_path.clone(),
    ))
}
