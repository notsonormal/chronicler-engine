use std::sync::Arc;

use crate::engine::game_service::context::GameServiceContext;
use crate::error::{EngineError, LlmFailure};
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;

/// [DOC: docs/architecture/system.md]
pub fn load_state(ctx: &GameServiceContext) -> GameState {
    match ctx.snapshot_storage.load_latest(None) {
        Ok(Some(snapshot)) => GameState::from_snapshot(
            &snapshot,
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).clone(),
        ),
        _ => GameState::new(
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).values().cloned().collect(),
            ctx.world.starting_room_id.clone(),
        ),
    }
}

/// [DOC: docs/architecture/system.md]
pub fn save_state(
    ctx: &GameServiceContext,
    state: &GameState,
    message_id: String,
    swipe_index: u32,
) {
    let snapshot = GameStateSnapshot::from_game_state(state, message_id, swipe_index);
    if let Err(e) = ctx.snapshot_storage.save(&snapshot) {
        log::error!("Failed to save snapshot: {e}");
    }
}

/// [DOC: docs/architecture/system.md]
pub fn save_committed_state(
    ctx: &GameServiceContext,
    state: &GameState,
    message_id: String,
    swipe_index: u32,
) {
    let mut snapshot = GameStateSnapshot::from_game_state(state, message_id, swipe_index);
    snapshot.committed = true;
    if let Err(e) = ctx.snapshot_storage.save(&snapshot) {
        log::error!("Failed to save committed snapshot: {e}");
    }
}

/// [DOC: docs/architecture/system.md]
pub fn map_llm_error(e: &EngineError) -> String {
    match e {
        EngineError::Llm(LlmFailure::Timeout) => "LLM Error: request timed out".to_string(),
        EngineError::Llm(LlmFailure::Network { url, detail }) => {
            format!("LLM Error: network error ({url}) — {detail}")
        }
        EngineError::Llm(LlmFailure::ParseError {
            expected_format, ..
        }) => {
            format!("LLM Error: unexpected response format (expected {expected_format})")
        }
        EngineError::Llm(LlmFailure::EmptyResponse) => "LLM Error: empty response".to_string(),
        EngineError::Llm(LlmFailure::Http { status, body }) => {
            format!("LLM Error: HTTP {status} — {body}")
        }
        EngineError::Narrative(nf) => format!("LLM Error: {nf}"),
        _ => format!("LLM Error: {e}"),
    }
}
