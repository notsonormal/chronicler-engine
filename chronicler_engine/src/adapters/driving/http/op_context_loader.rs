//! [DOC: docs/system/dashboard.md]
//! Loader for per-op application context

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::application::application_service::ApplicationError;
use crate::application::context::{OpContext, WorldSnapshot};
use crate::domain::model::character::NpcCard;
use crate::error::EngineError;

use super::app_state::AppState;

#[async_trait::async_trait]
impl FromRequestParts<AppState> for OpContext {
    type Rejection = ApplicationError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        load_op_context_for_active_game(state).map_err(ApplicationError::Engine)
    }
}

pub fn load_op_context(
    state: &AppState,
    world_key: &str,
    persona_key: &str,
) -> Result<OpContext, EngineError> {
    let world_with_map = state
        .storage
        .get_world(world_key)?
        .ok_or_else(|| EngineError::Config(format!("World not found: {world_key}")))?;

    let player = state
        .storage
        .get_persona(persona_key)?
        .ok_or_else(|| EngineError::Config(format!("Persona not found: {persona_key}")))?;

    let npcs = state.storage.list_characters(world_with_map.world_id)?;
    let npcs_map: HashMap<String, NpcCard> =
        npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect();

    Ok(OpContext {
        storage: Arc::clone(&state.storage),
        world_snapshot: WorldSnapshot {
            world: Arc::new(world_with_map.world_card),
            map: Arc::new(world_with_map.map),
            player: Arc::new(player),
            npcs: Arc::new(npcs_map),
        },
        cancel_token: state.current_cancel_token(),
        is_generating: Arc::clone(&state.is_generating),
        settings: Arc::clone(&state.settings),
        preset_storage: Arc::clone(&state.preset_storage),
    })
}

pub fn load_op_context_for_active_game(state: &AppState) -> Result<OpContext, EngineError> {
    let game_id = state.storage.current_game_id();
    let game = state
        .storage
        .get_game(game_id)?
        .ok_or_else(|| EngineError::Config("No active game".to_string()))?;
    load_op_context(state, &game.world_key, &game.persona_key)
}
