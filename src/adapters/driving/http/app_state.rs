//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Application state and HTTP fragment rendering.

use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use askama::Template;

use crate::adapters::driving::http::builders::headers::render_header_unlocked;
use crate::adapters::driving::http::templates::{
    ActionAreaTemplate, CharacterHeadshotsTemplate, LlmMessagesTemplate, NarrativeLogTemplate,
    VisualSidebarTemplate,
};
use crate::adapters::driving::http::view_models::{
    ActionAreaViewModel, NpcPortraitView, VisualSidebarViewModel,
};
use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GenerationGate;
use crate::application::message_service::MessageService;
use crate::application::persona_catalogue::PersonaCatalogue;
use crate::application::pipeline::ActionPipeline;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::settings_service::SettingsService;
use crate::application::text_check_service::TextCheckService;
use crate::application::world_catalogue::WorldCatalogue;
use crate::bootstrap::wiring::WiredApp;
use crate::domain::model::settings::AppSettings;
use crate::error::{EngineError, Result};

use super::utils::read_lock_or_recover;

#[derive(Clone)]
pub struct AppState {
    pub settings_service: SettingsService,
    pub prompt_preset_service: PromptPresetService,
    pub message_service: Arc<MessageService>,
    pub world_catalogue: WorldCatalogue,
    pub persona_catalogue: PersonaCatalogue,
    pub text_check_service: Arc<TextCheckService>,
    pub settings: Arc<std::sync::RwLock<AppSettings>>,
    pub shutdown_token: CancellationToken,
    pub pipeline: Arc<ActionPipeline>,
    pub generation_gate: GenerationGate,
    pub game_catalogue: GameCatalogue,
    pub game_view_query: GameViewQuery,
}

impl AppState {
    pub fn from_wired(wired: WiredApp) -> Self {
        AppState {
            settings_service: wired.settings_service,
            prompt_preset_service: wired.prompt_preset_service,
            message_service: wired.message_service,
            world_catalogue: wired.world_catalogue,
            persona_catalogue: wired.persona_catalogue,
            text_check_service: wired.text_check_service,
            settings: wired.settings,
            shutdown_token: wired.shutdown_token,
            pipeline: Arc::new(wired.pipeline),
            generation_gate: wired.generation_gate,
            game_catalogue: wired.game_catalogue,
            game_view_query: wired.game_view_query,
        }
    }

    pub fn current_shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    pub fn settings(&self) -> AppSettings {
        read_lock_or_recover(&self.settings, "settings")
    }

    pub fn text_check_service(&self) -> &TextCheckService {
        &self.text_check_service
    }

    fn render_error_context(name: &str, e: impl std::fmt::Display) -> String {
        format!("Failed to render {name}: {e}")
    }

    pub fn render_header(&self) -> Result<String> {
        let game_name = self
            .game_view_query
            .get_current_game_name()
            .unwrap_or_else(|_| "Unknown".to_string());
        render_header_unlocked(game_name)
    }

    pub fn render_story_log(&self) -> Result<String> {
        const MAX_LOG_DISPLAY: usize = 50;

        let (entries, has_last_trigger) = self
            .game_view_query
            .get_story_log_entries()
            .map_err(|e| EngineError::Config(Self::render_error_context("story log", e)))?;

        let entries: Vec<_> = entries.into_iter().take(MAX_LOG_DISPLAY).collect();
        let template = NarrativeLogTemplate::new(&entries, has_last_trigger);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::render_error_context("story log", e)))
    }

    pub fn render_visual_sidebar(&self) -> Result<String> {
        let (room_name, image_path) = self
            .game_view_query
            .get_current_room_view()
            .map_err(|e| EngineError::Config(Self::render_error_context("visual sidebar", e)))?;

        let npc_data = self
            .game_view_query
            .get_npc_headshots(true)
            .map_err(|e| EngineError::Config(Self::render_error_context("visual sidebar", e)))?;

        let npc_portraits: Vec<NpcPortraitView> = npc_data
            .into_iter()
            .map(|(image_path, name)| NpcPortraitView { image_path, name })
            .collect();

        let vm = VisualSidebarViewModel::new(image_path, room_name, npc_portraits);
        let template = VisualSidebarTemplate::new(vm);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::render_error_context("visual sidebar", e)))
    }

    pub fn render_action_area(&self) -> Result<String> {
        let (status, phase) = self
            .game_view_query
            .get_generating_status()
            .map_err(|e| EngineError::Config(Self::render_error_context("action area", e)))?;

        let vm = ActionAreaViewModel::new(&status, &phase);
        let template = ActionAreaTemplate::new(vm);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::render_error_context("action area", e)))
    }

    pub fn render_character_headshots(&self) -> Result<String> {
        let npc_data = self.game_view_query.get_npc_headshots(false).map_err(|e| {
            EngineError::Config(Self::render_error_context("character headshots", e))
        })?;

        let npc_portraits: Vec<NpcPortraitView> = npc_data
            .into_iter()
            .map(|(image_path, name)| NpcPortraitView { image_path, name })
            .collect();

        let template = CharacterHeadshotsTemplate::new(npc_portraits);
        template.render().map_err(|e| {
            EngineError::Template(Self::render_error_context("character headshots", e))
        })
    }

    pub fn render_llm_messages(&self) -> Result<String> {
        let messages = self
            .game_view_query
            .list_latest_llm_messages(50)
            .map_err(|e| EngineError::Config(Self::render_error_context("LLM messages", e)))?;

        let template = LlmMessagesTemplate::new(&messages);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::render_error_context("LLM messages", e)))
    }
}
