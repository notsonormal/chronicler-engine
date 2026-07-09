//! [DOC: docs/system/dashboard.md]
//! Application state management

use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::GameService;
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::settings::AppSettings;

use super::locks::{read_lock_or_recover, write_lock_or_recover};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub bind_attempts: Option<u32>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: 3000,
            bind_attempts: None,
        }
    }
}

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
    pub cancel_token: Arc<std::sync::RwLock<CancellationToken>>,
    pub is_generating: Arc<AtomicBool>,
}

impl AppState {
    pub fn current_cancel_token(&self) -> CancellationToken {
        read_lock_or_recover(&self.cancel_token, "cancel_token")
    }

    pub fn replace_cancel_token(&self) {
        let mut token = write_lock_or_recover(&self.cancel_token, "cancel_token");
        *token = CancellationToken::new();
    }

    pub fn settings(&self) -> AppSettings {
        read_lock_or_recover(&self.settings, "settings")
    }

    pub fn text_check_service(&self) -> &TextCheckService {
        &self.text_check_service
    }
}
