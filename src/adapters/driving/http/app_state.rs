//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Application state and HTTP fragment rendering.

use std::sync::Arc;

use askama::Template;

use crate::adapters::driving::http::builders::headers::{failure_banner_html, header_fragment_html};
use crate::adapters::driving::http::templates::{
    LlmMessageRecordTemplate, LlmMessagesTemplate, NarrationLogTemplate, OptionsDockTemplate,
    VisualSidebarTemplate,
};
use crate::adapters::driving::http::view_models::{
    CharacterPortraitView, OptionsDockViewModel, VisualSidebarViewModel,
};
use crate::application::connection_test_service::ConnectionTestService;
use crate::application::errors::ApplicationError;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GameWriter;
use crate::application::persona_catalogue::PersonaCatalogue;
use crate::application::pipeline::job::JobDeps;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::settings_service::SettingsService;
use crate::application::text_check_service::TextCheckService;
use crate::application::world_catalogue::WorldCatalogue;
use crate::bootstrap::wiring::WiredApp;
use crate::domain::model::game::GameId;
use crate::domain::model::settings::AppSettings;
use crate::error::{EngineError, Result};

#[derive(Clone)]
pub struct AppState {
    pub settings_service: SettingsService,
    pub prompt_preset_service: PromptPresetService,
    pub world_catalogue: WorldCatalogue,
    pub persona_catalogue: PersonaCatalogue,
    pub text_check_service: Arc<TextCheckService>,
    pub connection_test_service: ConnectionTestService,
    pub game_writer: GameWriter,
    pub job_deps: JobDeps,
    pub game_catalogue: GameCatalogue,
    pub game_view_query: GameViewQuery,
}

impl AppState {
    pub fn from_wired(wired: WiredApp) -> Self {
        AppState {
            settings_service: wired.settings_service,
            prompt_preset_service: wired.prompt_preset_service,
            world_catalogue: wired.world_catalogue,
            persona_catalogue: wired.persona_catalogue,
            text_check_service: wired.text_check_service,
            connection_test_service: wired.connection_test_service,
            game_writer: wired.game_writer,
            job_deps: wired.job_deps,
            game_catalogue: wired.game_catalogue,
            game_view_query: wired.game_view_query,
        }
    }

    pub fn settings(&self) -> Result<AppSettings> {
        self.settings_service.get_settings()
    }

    #[cfg(feature = "testing")]
    pub fn with_connection_test_provider(
        mut self,
        provider_factory: crate::application::connection_test_service::ProviderFactory,
    ) -> Self {
        self.connection_test_service = self
            .connection_test_service
            .with_provider_factory(provider_factory);
        self
    }

    pub fn text_check_service(&self) -> &TextCheckService {
        &self.text_check_service
    }

    fn template_failure_context(name: &str, e: impl std::fmt::Display) -> String {
        format!("Failed to render {name}: {e}")
    }

    pub fn render_header(&self, game_id: GameId) -> Result<String> {
        let game_name = self
            .game_view_query
            .game_display_name(game_id)
            .unwrap_or_else(|error| {
                tracing::warn!("header: the game name could not be read: {error}");
                "Unknown".to_string()
            });
        header_fragment_html(game_name)
    }

    pub fn render_failure_banner(&self) -> Result<String> {
        let roles = self
            .game_view_query
            .role_health()
            .map_err(ApplicationError::into_engine_error)?;
        Ok(failure_banner_html(&roles))
    }

    pub fn render_story_log(&self, game_id: GameId) -> Result<String> {
        let window = self
            .game_view_query
            .story_window(game_id)
            .map_err(ApplicationError::into_engine_error)?;

        let view_only = self.game_view_query.autoplay_holds(game_id);
        NarrationLogTemplate::new(&window.entries, window.retrigger_allowed, view_only)
            .render()
            .map_err(|e| EngineError::Template(Self::template_failure_context("story log", e)))
    }

    pub fn render_visual_sidebar(&self, game_id: GameId) -> Result<String> {
        let (room_name, image_path) = self
            .game_view_query
            .get_current_room_view(game_id)
            .map_err(ApplicationError::into_engine_error)?;

        let character_data = self
            .game_view_query
            .get_character_headshots(game_id)
            .map_err(ApplicationError::into_engine_error)?;

        let character_portraits: Vec<CharacterPortraitView> = character_data
            .into_iter()
            .map(|(image_path, name)| CharacterPortraitView { image_path, name })
            .collect();

        let vm = VisualSidebarViewModel::new(image_path, room_name, character_portraits);
        let template = VisualSidebarTemplate::new(vm);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::template_failure_context("visual sidebar", e)))
    }

    pub fn render_options_dock(&self, game_id: GameId) -> Result<String> {
        let dock = self
            .game_view_query
            .dock(game_id)
            .map_err(ApplicationError::into_engine_error)?;
        // The one registry, not a saved record, answers "is something generating?".
        let is_busy = self.game_view_query.is_busy(game_id);

        let vm = OptionsDockViewModel::new(dock.offered, is_busy, dock.failure);
        let template = OptionsDockTemplate::new(vm);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::template_failure_context("options dock", e)))
    }

    pub fn render_llm_messages(&self) -> Result<String> {
        let messages = self
            .game_view_query
            .list_latest_llm_messages(50)
            .map_err(ApplicationError::into_engine_error)?;

        let template = LlmMessagesTemplate::new(&messages);
        template
            .render()
            .map_err(|e| EngineError::Template(Self::template_failure_context("LLM messages", e)))
    }

    /// `None` when no record has this id: retention removes the oldest records.
    pub fn render_llm_message_record(&self, id: i64) -> Result<Option<String>> {
        let message = self
            .game_view_query
            .llm_message(id)
            .map_err(ApplicationError::into_engine_error)?;
        message
            .map(|message| {
                LlmMessageRecordTemplate::new(&message)
                    .render()
                    .map_err(|e| {
                        EngineError::Template(Self::template_failure_context("LLM message", e))
                    })
            })
            .transpose()
    }
}
