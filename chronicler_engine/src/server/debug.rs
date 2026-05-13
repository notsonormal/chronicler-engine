use std::collections::HashMap;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::model::state::{GenerationPhase, GenerationStatus, LogEntry};
use crate::model::trigger::NpcEncounterState;
use crate::server::AppState;

#[derive(Serialize)]
pub struct DebugStateResponse {
    pub current_room_id: String,
    pub npcs_in_area: Vec<String>,
    pub generation_status: GenerationStatus,
    pub generation_phase: GenerationPhase,
    pub character_state: HashMap<String, NpcEncounterState>,
    pub narration_history_tail: Vec<LogEntry>,
    pub narration_history_length: usize,
    pub dynamic_rooms: Vec<String>,
    pub dynamic_room_count: usize,
    pub last_error: Option<String>,
}

/// NOTE: dev-only diagnostic endpoint
/// [DOC: docs/system/game_flow.md]
pub async fn debug_state_handler(
    State(state): State<AppState>,
) -> Result<Json<DebugStateResponse>, StatusCode> {
    let guard = match state.load_state() {
        Ok(g) => g,
        Err(_) => {
            log::error!("State load failed during /debug/state request");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Only take the last 5 entries to keep the response scannable
    let history_tail: Vec<LogEntry> = guard
        .narrative
        .history()
        .iter()
        .rev()
        .take(5)
        .rev()
        .cloned()
        .collect();

    let npcs_in_area: Vec<String> = guard
        .scene
        .npcs_in_area
        .iter()
        .map(|npc| npc.id.clone())
        .collect();

    let dynamic_rooms: Vec<String> = guard.movement.dynamic_rooms.keys().cloned().collect();

    let last_error = match &guard.narrative.generation.status {
        GenerationStatus::Error(msg) => Some(msg.clone()),
        _ => None,
    };

    let response = DebugStateResponse {
        current_room_id: guard.movement.current_room_id.clone(),
        npcs_in_area,
        generation_status: guard.narrative.generation.status.clone(),
        generation_phase: guard.narrative.generation.phase.clone(),
        character_state: guard.character_state.npcs.clone(),
        narration_history_tail: history_tail,
        narration_history_length: guard.narrative.history().len(),
        dynamic_rooms,
        dynamic_room_count: guard.movement.dynamic_rooms.len(),
        last_error,
    };

    Ok(Json(response))
}
