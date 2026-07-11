//! [DOC: docs/architecture/invariants.md]
//! State diagnostics and debugging utilities

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::EngineError;
#[cfg(feature = "diagnostics")]
use crate::error::internal_error;
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::game_state::GameState;

#[cfg(feature = "diagnostics")]
pub fn assert_state_consistency(
    state: &GameState,
    map: &Arc<MapDef>,
    npcs: &HashMap<String, NpcCard>,
) -> Result<(), EngineError> {
    assert_room_exists(state, map)?;
    assert_npc_consistency(state, npcs)?;
    assert_npc_encounter_log_consistency(state, npcs)?;
    assert_log_invariants(state)?;
    Ok(())
}

#[cfg(not(feature = "diagnostics"))]
pub fn assert_state_consistency(
    _state: &GameState,
    _map: &Arc<MapDef>,
    _npcs: &HashMap<String, NpcCard>,
) -> Result<(), EngineError> {
    Ok(())
}

/// INV-ROOM: current_room_id must exist in the map or dynamic_rooms.
#[cfg(feature = "diagnostics")]
fn assert_room_exists(state: &GameState, map: &Arc<MapDef>) -> Result<(), EngineError> {
    let current_room = map
        .get_room_by_id(&state.movement.current_room_id)
        .or_else(|| {
            state
                .movement
                .dynamic_rooms
                .get(&state.movement.current_room_id)
        });
    if current_room.is_none() {
        return Err(EngineError::Internal(internal_error(format!(
            "current_room_id '{}' not found in map or dynamic_rooms",
            state.movement.current_room_id
        ))));
    }
    Ok(())
}

/// INV-NPC: every NPC in npcs_in_area must exist in the global npcs map.
#[cfg(feature = "diagnostics")]
fn assert_npc_consistency(
    state: &GameState,
    npcs: &HashMap<String, NpcCard>,
) -> Result<(), EngineError> {
    for npc in &state.scene.npcs_in_area {
        if !npcs.contains_key(&npc.id) {
            return Err(EngineError::Internal(internal_error(format!(
                "npcs_in_area contains NPC '{}' which is not in the npcs map",
                npc.id
            ))));
        }
    }
    Ok(())
}

/// INV-CHAR: npc_encounter_log entries must reference valid NPCs.
#[cfg(feature = "diagnostics")]
fn assert_npc_encounter_log_consistency(
    state: &GameState,
    npcs: &HashMap<String, NpcCard>,
) -> Result<(), EngineError> {
    for npc_id in state.npc_encounter_log.npcs.keys() {
        if !npcs.contains_key(npc_id) {
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
    let ai_idx = state.narrative.history.last_ai_response_index();
    let input_idx = state.narrative.history.last_input_index();

    if let (Some(ai), Some(input)) = (ai_idx, input_idx) {
        if ai <= input {
            return Err(EngineError::Internal(internal_error(
                "last AI response is not after last player input",
            )));
        }
    }
    Ok(())
}
