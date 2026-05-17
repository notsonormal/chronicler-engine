//! Runtime consistency checks for GameState.
//! [DOC: docs/architecture/invariants.md]

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
    assert_npc_encounter_log_consistency(state)?;
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
    if state.current_room().is_none() {
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

/// INV-CHAR: npc_encounter_log entries must reference valid NPCs.
#[cfg(feature = "diagnostics")]
fn assert_npc_encounter_log_consistency(state: &GameState) -> Result<(), EngineError> {
    for npc_id in state.npc_encounter_log.npcs.keys() {
        if !state.npcs.contains_key(npc_id) {
            return Err(EngineError::Internal(internal_error(format!(
                "npc_encounter_log references unknown NPC '{npc_id}'"
            ))));
        }
    }
    Ok(())
}

/// INV-LOG: the last AI response must follow the last player input.
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
