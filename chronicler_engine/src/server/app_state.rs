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
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::AppSettings;

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
    pub storage: Arc<crate::storage::Storage>,
    pub preset_storage: Arc<crate::storage::Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
}

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<crate::storage::Storage>,
    pub preset_storage: Arc<crate::storage::Storage>,
    pub game_service: Arc<GameService>,
    pub application_service: Arc<DefaultApplicationService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: Arc<std::sync::RwLock<CancellationToken>>,
    pub is_generating: Arc<AtomicBool>,
}

impl AppState {
    /// Constructs a `GameServiceContext` from this state by loading world data from DB.
    pub fn as_game_service_context(&self) -> Result<GameServiceContext, EngineError> {
        let game_id = self.storage.current_game_id();
        let game = self
            .storage
            .get_game(game_id)?
            .ok_or_else(|| EngineError::Config("No active game".to_string()))?;
        let world_with_map = self
            .storage
            .get_world(&game.world_key)?
            .ok_or_else(|| EngineError::Config(format!("World not found: {}", game.world_key)))?;
        let player = self
            .storage
            .get_persona(&world_with_map.world_card.player_key)?
            .ok_or_else(|| {
                EngineError::Config(format!(
                    "Persona not found: {}",
                    world_with_map.world_card.player_key
                ))
            })?;
        let npcs = self.storage.list_characters(world_with_map.world_id)?;
        let npcs_map: HashMap<String, NpcCard> =
            npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect();

        Ok(GameServiceContext {
            storage: Arc::clone(&self.storage),
            world: Arc::new(world_with_map.world_card),
            map: Arc::new(world_with_map.map),
            player: Arc::new(player),
            npcs: Arc::new(npcs_map),
            cancel_token: self.current_cancel_token(),
            is_generating: Arc::clone(&self.is_generating),
            settings: Arc::clone(&self.settings),
            preset_storage: Arc::clone(&self.preset_storage),
        })
    }

    // Note: Removed as_game_service_context_or_default() - see ADR-025.
    // Callers should use as_game_service_context() and propagate errors properly.
    // Silent error swallowing led to blank pages with no indication of DB corruption
    // or missing world data. All handlers now return 500 errors on context failures.

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
