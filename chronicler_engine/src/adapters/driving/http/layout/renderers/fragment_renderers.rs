//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Fragment renderers

use askama::Template;
use crate::error::{EngineError, Result};
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::headers::render_header_unlocked;
use crate::adapters::driving::http::templates::{
    ActionAreaTemplate, LlmMessagesTemplate, NarrativeLogTemplate, VisualSidebarTemplate,
};
use crate::adapters::driving::http::view_models::{
    ActionAreaViewModel, NpcPortraitView, VisualSidebarViewModel,
};

const MAX_LOG_DISPLAY: usize = 50;

impl AppState {
    pub fn render_header(&self) -> Result<String> {
        let game_name = self
            .game_view_query
            .get_current_game_name()
            .unwrap_or_else(|_| "Unknown".to_string());
        render_header_unlocked(game_name)
    }

    pub fn render_story_log(&self) -> Result<String> {
        let (entries, has_last_trigger) = self
            .game_view_query
            .get_story_log_entries()
            .map_err(|e| EngineError::Config(e.to_string()))?;

        let entries: Vec<_> = entries.into_iter().take(MAX_LOG_DISPLAY).collect();
        let template = NarrativeLogTemplate::new(&entries, has_last_trigger);
        template
            .render()
            .map_err(|e| EngineError::Template(e.to_string()))
    }

    pub fn render_visual_sidebar(&self) -> Result<String> {
        let (room_name, image_path) = self
            .game_view_query
            .get_current_room_view()
            .map_err(|e| EngineError::Config(e.to_string()))?;

        let npc_data = self
            .game_view_query
            .get_npc_headshots(true)
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

    pub fn render_action_area(&self) -> Result<String> {
        let (status, phase) = self
            .game_view_query
            .get_generating_status()
            .map_err(|e| EngineError::Config(e.to_string()))?;

        let vm = ActionAreaViewModel::new(&status, &phase);
        let template = ActionAreaTemplate::new(vm);
        template
            .render()
            .map_err(|e| EngineError::Template(e.to_string()))
    }

    pub fn render_character_headshots(&self) -> Result<String> {
        use crate::adapters::driving::http::templates::CharacterHeadshotsTemplate;
        use askama::Template;

        let npc_data = self
            .game_view_query
            .get_npc_headshots(false)
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

    pub fn render_llm_messages(&self) -> Result<String> {
        let messages = self
            .game_view_query
            .list_latest_llm_messages(50)
            .map_err(|e| EngineError::Config(e.to_string()))?;

        let template = LlmMessagesTemplate::new(&messages);
        template
            .render()
            .map_err(|e| EngineError::Template(e.to_string()))
    }
}
