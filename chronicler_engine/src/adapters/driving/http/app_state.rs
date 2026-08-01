//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Application state management

use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GenerationGate;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::pipeline::pipeline::ActionPipeline;
use crate::application::text_check_service::TextCheckService;
use crate::bootstrap::wiring::WiredApp;
use crate::domain::model::settings::AppSettings;

use super::utils::read_lock_or_recover;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<crate::adapters::driven::storage::Storage>,
    pub preset_storage: Arc<crate::adapters::driven::storage::Storage>,
    pub persistence_gate: Arc<PersistenceGate>,
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
            storage: wired.storage,
            preset_storage: wired.preset_storage,
            persistence_gate: wired.persistence_gate,
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
}
