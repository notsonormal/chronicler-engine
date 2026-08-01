//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Application state management

use std::sync::Arc;
use std::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::text_check_service::TextCheckService;
use crate::bootstrap::wiring::WiredApp;
use crate::domain::model::settings::AppSettings;

use super::utils::read_lock_or_recover;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<crate::adapters::driven::storage::Storage>,
    pub preset_storage: Arc<crate::adapters::driven::storage::Storage>,
    pub application_service: Arc<DefaultApplicationService>,
    pub persistence_gate: Arc<PersistenceGate>,
    pub text_check_service: Arc<TextCheckService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub shutdown_token: Arc<std::sync::RwLock<CancellationToken>>,
}

impl AppState {
    pub fn from_wired(wired: WiredApp, shutdown_token: CancellationToken) -> Self {
        AppState {
            storage: wired.storage,
            preset_storage: wired.preset_storage,
            application_service: wired.application_service,
            persistence_gate: wired.persistence_gate,
            text_check_service: wired.text_check_service,
            settings: wired.settings,
            shutdown_token: Arc::new(RwLock::new(shutdown_token)),
        }
    }

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
