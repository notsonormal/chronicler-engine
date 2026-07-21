//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Application state management

use std::sync::Arc;
use std::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::GameService;
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::settings::AppSettings;

use super::locks::read_lock_or_recover;

#[derive(Clone)]
pub struct ServerResources {
    pub storage: Arc<crate::adapters::driven::storage::Storage>,
    pub preset_storage: Arc<crate::adapters::driven::storage::Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub game_service: Arc<GameService>,
    pub text_check_service: Arc<TextCheckService>,
}

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<crate::adapters::driven::storage::Storage>,
    pub preset_storage: Arc<crate::adapters::driven::storage::Storage>,
    pub game_service: Arc<GameService>,
    pub application_service: Arc<DefaultApplicationService>,
    pub text_check_service: Arc<TextCheckService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub shutdown_token: Arc<std::sync::RwLock<CancellationToken>>,
}

impl AppState {
    pub fn current_shutdown_token(&self) -> CancellationToken {
        read_lock_or_recover(&self.shutdown_token, "shutdown_token")
    }

    pub fn settings(&self) -> AppSettings {
        read_lock_or_recover(&self.settings, "settings")
    }

    pub fn text_check_service(&self) -> &TextCheckService {
        &self.text_check_service
    }
}
