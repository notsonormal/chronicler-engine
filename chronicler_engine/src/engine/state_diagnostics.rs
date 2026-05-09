//! Runtime consistency checks for GameState.
//!
//! These checks are enabled via the `diagnostics` feature flag.
//! They verify load-bearing invariants after state mutations.
//! [DOC: docs/architecture/invariants.md]

#[cfg(feature = "diagnostics")]
use crate::engine::logic::get_current_room;
use crate::error::EngineError;
#[cfg(feature = "diagnostics")]
use crate::error::internal_error;
use crate::model::state::GameState;

/// Run all consistency checks on GameState.
///
/// Call this after any public mutation function to catch invariant
/// violations immediately at the site of the bug.
#[cfg(feature = "diagnostics")]
pub fn assert_state_consistency(state: &GameState) -> Result<(), EngineError> {
    // [DOC: docs/architecture/invariants.md]
    assert_room_exists(state)?;
    assert_npc_consistency(state)?;
    assert_character_state_consistency(state)?;
    assert_log_invariants(state)?;
    Ok(())
}

/// No-op when diagnostics feature is disabled.
#[cfg(not(feature = "diagnostics"))]
pub fn assert_state_consistency(_state: &GameState) -> Result<(), EngineError> {
    Ok(())
}

/// INV-ROOM: current_room_id must exist in the map or dynamic_rooms.
#[cfg(feature = "diagnostics")]
fn assert_room_exists(state: &GameState) -> Result<(), EngineError> {
    if get_current_room(state).is_err() {
        return Err(EngineError::Internal(internal_error(format!(
            "current_room_id '{}' not found in map or dynamic_rooms",
            state.movement.current_room_id
        ))));
    }
    Ok(())
}

/// INV-NPC: every NPC in npcs_in_area must exist in the global npcs map.
#[cfg(feature = "diagnostics")]
fn assert_npc_consistency(state: &GameState) -> Result<(), EngineError> {
    for npc in &state.scene.npcs_in_area {
        if !state.npcs.contains_key(&npc.id) {
            return Err(EngineError::Internal(internal_error(format!(
                "npcs_in_area contains NPC '{}' which is not in state.npcs",
                npc.id
            ))));
        }
    }
    Ok(())
}

/// INV-CHAR: character_state entries must reference valid NPCs.
#[cfg(feature = "diagnostics")]
fn assert_character_state_consistency(state: &GameState) -> Result<(), EngineError> {
    for npc_id in state.character_state.npcs.keys() {
        if !state.npcs.contains_key(npc_id) {
            return Err(EngineError::Internal(internal_error(format!(
                "character_state references unknown NPC '{npc_id}'"
            ))));
        }
    }
    Ok(())
}

/// INV-LOG: the last AI response must follow the last player input.
/// This is the load-bearing invariant for `replace_last_ai_response`.
#[cfg(feature = "diagnostics")]
fn assert_log_invariants(state: &GameState) -> Result<(), EngineError> {
    let ai_idx = state.get_last_ai_response_index();
    let input_idx = state.get_last_input_index();

    if let (Some(ai), Some(input)) = (ai_idx, input_idx) {
        if ai <= input {
            return Err(EngineError::Internal(internal_error(
                "last AI response is not after last player input",
            )));
        }
    }
    Ok(())
}
