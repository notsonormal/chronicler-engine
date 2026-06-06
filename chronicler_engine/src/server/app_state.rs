//! [DOC: docs/system/dashboard.md]
//! Application state management

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::GameService;
use crate::application::GameServiceContext;
use crate::model::character::NpcCard;
use crate::model::character::PlayerCard;
use crate::model::map::MapDef;
use crate::model::settings::AppSettings;
use crate::model::world::WorldCard;

/// Server configuration.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: 3000 }
    }
}

/// Resources required to initialize the server.
#[derive(Clone)]
pub struct ServerResources {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub storage: Arc<crate::storage::Storage>,
    pub preset_storage: Arc<crate::storage::Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
}

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<crate::storage::Storage>,
    pub preset_storage: Arc<crate::storage::Storage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub game_service: Arc<GameService>,
    pub application_service: Arc<DefaultApplicationService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: Arc<std::sync::RwLock<CancellationToken>>,
    pub is_generating: Arc<AtomicBool>,
}

impl AppState {
    /// Constructs a `GameServiceContext` from this state.
    pub fn as_game_service_context(&self) -> GameServiceContext {
        GameServiceContext {
            storage: Arc::clone(&self.storage),
            world: Arc::clone(&self.world),
            map: Arc::clone(&self.map),
            player: Arc::clone(&self.player),
            npcs: Arc::clone(&self.npcs),
            cancel_token: self.current_cancel_token(),
            is_generating: Arc::clone(&self.is_generating),
            settings: Arc::clone(&self.settings),
            preset_storage: Arc::clone(&self.preset_storage),
        }
    }

    /// If the lock is poisoned, recovers the inner value.
    pub fn current_cancel_token(&self) -> CancellationToken {
        self.cancel_token
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|p| {
                tracing::warn!("Poisoned cancel_token read lock recovered");
                p.into_inner().clone()
            })
    }

    /// If the lock is poisoned, recovers the inner value before replacing.
    pub fn replace_cancel_token(&self) {
        let mut token = self.cancel_token.write().unwrap_or_else(|p| {
            tracing::warn!("Poisoned cancel_token write lock recovered");
            p.into_inner()
        });
        *token = CancellationToken::new();
    }

    /// If the lock is poisoned, recovers the inner value.
    pub fn settings(&self) -> AppSettings {
        self.settings.read().map(|g| g.clone()).unwrap_or_else(|p| {
            tracing::warn!("Poisoned settings read lock recovered");
            p.into_inner().clone()
        })
    }
}
