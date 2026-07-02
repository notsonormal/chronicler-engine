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
use crate::application::text_check_service::TextCheckService;
use crate::error::EngineError;

use super::locks::{read_lock_or_recover, write_lock_or_recover};
use crate::domain::model::character::NpcCard;
use crate::domain::model::settings::AppSettings;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: 3000 }
    }
}

#[derive(Clone)]
pub struct ServerResources {
    pub storage: Arc<crate::adapters::driven::storage::Storage>,
    pub preset_storage: Arc<crate::adapters::driven::storage::Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
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
    pub fn context_for_world(
        &self,
        world_key: &str,
        persona_key: &str,
    ) -> Result<GameServiceContext, EngineError> {
        let world_with_map = self
            .storage
            .get_world(world_key)?
            .ok_or_else(|| EngineError::Config(format!("World not found: {world_key}")))?;

        let player = self
            .storage
            .get_persona(persona_key)?
            .ok_or_else(|| EngineError::Config(format!("Persona not found: {persona_key}")))?;

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

    pub fn as_game_service_context(&self) -> Result<GameServiceContext, EngineError> {
        let game_id = self.storage.current_game_id();
        let game = self
            .storage
            .get_game(game_id)?
            .ok_or_else(|| EngineError::Config("No active game".to_string()))?;
        self.context_for_world(&game.world_key, &game.persona_key)
    }

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
