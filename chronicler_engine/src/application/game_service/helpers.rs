use std::sync::Arc;

use crate::application::game_service::context::GameServiceContext;
use crate::error::{EngineError, LlmFailure};
use crate::model::state::GameState;
use crate::model::state_snapshot::GameStateSnapshot;

/// [DOC: docs/architecture/system.md]
pub fn load_state(ctx: &GameServiceContext) -> GameState {
    match ctx.snapshot_storage.load_latest() {
        Ok(Some(snapshot)) => {
            let mut state = GameState::from_snapshot(
                &snapshot,
                Arc::clone(&ctx.world),
                Arc::clone(&ctx.map),
                Arc::clone(&ctx.player),
                (*ctx.npcs).clone(),
            );
            load_messages_into_state(ctx, &mut state);
            state
        }
        _ => GameState::new(
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).values().cloned().collect(),
            ctx.world.starting_room_id.clone(),
        ),
    }
}

pub fn load_messages_into_state(ctx: &GameServiceContext, state: &mut GameState) {
    // [DOC: docs/architecture/system.md]
    if let Ok(msgs) = ctx.message_storage.load_messages() {
        state.narrative.messages = msgs;
    }
}

/// [DOC: docs/architecture/system.md]
pub fn save_state(ctx: &GameServiceContext, state: &mut GameState) -> Result<u64, EngineError> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    save_snapshot(ctx, state, snapshot)
}

/// [DOC: docs/architecture/system.md]
pub fn save_committed_state(
    ctx: &GameServiceContext,
    state: &mut GameState,
) -> Result<u64, EngineError> {
    let mut snapshot = GameStateSnapshot::from_game_state(state);
    snapshot.committed = true;
    save_snapshot(ctx, state, snapshot)
}

fn save_snapshot(
    ctx: &GameServiceContext,
    state: &mut GameState,
    snapshot: GameStateSnapshot,
) -> Result<u64, EngineError> {
    let snapshot_id = ctx.snapshot_storage.save(&snapshot)?;
    persist_new_messages(ctx, state, snapshot_id)?;
    Ok(snapshot_id)
}

/// Persist any messages with `id == 0` (newly added) to storage, assigning
/// them real DB IDs and `snapshot_id`.
pub fn persist_new_messages(
    ctx: &GameServiceContext,
    state: &mut GameState,
    snapshot_id: u64,
) -> Result<(), EngineError> {
    // [DOC: docs/architecture/system.md]
    for msg in state.narrative.messages.iter_mut() {
        if msg.id == crate::model::storage::UNPERSISTED_ID {
            msg.snapshot_id = Some(snapshot_id);
            ctx.message_storage.insert_message(msg)?;
        }
    }
    Ok(())
}

pub fn delete_and_remove_message(
    ctx: &GameServiceContext,
    state: &mut GameState,
    id: u64,
) -> Result<(), EngineError> {
    // [DOC: docs/architecture/system.md]
    ctx.message_storage.delete_message(id)?;
    state.narrative.messages.retain(|m| m.id != id);
    Ok(())
}

/// [DOC: docs/architecture/system.md]
pub fn map_llm_error(e: &EngineError) -> String {
    match e {
        EngineError::Llm(LlmFailure::Timeout) => "LLM Error: request timed out".to_string(),
        EngineError::Llm(LlmFailure::Network { url, detail }) => {
            format!("LLM Error: network error ({url}) \u{2014} {detail}")
        }
        EngineError::Llm(LlmFailure::ParseError {
            expected_format, ..
        }) => {
            format!("LLM Error: unexpected response format (expected {expected_format})")
        }
        EngineError::Llm(LlmFailure::EmptyResponse) => "LLM Error: empty response".to_string(),
        EngineError::Llm(LlmFailure::Http { status, body }) => {
            format!("LLM Error: HTTP {status} \u{2014} {body}")
        }
        EngineError::Narrative(nf) => format!("LLM Error: {nf}"),
        _ => format!("LLM Error: {e}"),
    }
}
