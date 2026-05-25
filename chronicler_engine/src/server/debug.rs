use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::server::AppState;

#[derive(Serialize)]
pub struct DebugStateResponse {
    pub current_room_id: String,
    pub npcs_in_area: Vec<String>,
    pub generation_status: String,
    pub generation_phase: String,
    pub npc_encounter_log: std::collections::HashMap<String, serde_json::Value>,
    pub narration_history_tail: Vec<serde_json::Value>,
    pub narration_history_length: usize,
    pub dynamic_rooms: Vec<String>,
    pub dynamic_room_count: usize,
    pub last_error: Option<String>,
    pub quantifier_confidence: Option<String>,
    pub backend_name: Option<String>,
    pub model_name: Option<String>,
}

/// NOTE: dev-only diagnostic endpoint
/// [DOC: docs/system/game_flow.md]
pub async fn debug_state_handler(
    State(state): State<AppState>,
) -> Result<Json<DebugStateResponse>, StatusCode> {
    let view = match state
        .application_service
        .get_debug_state(state.as_game_service_context())
    {
        Ok(v) => v,
        Err(_) => {
            log::error!("State load failed during /debug/state request");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = DebugStateResponse {
        current_room_id: view.current_room_id,
        npcs_in_area: view.npcs_in_area,
        generation_status: format!("{:?}", view.generation_status),
        generation_phase: format!("{:?}", view.generation_phase),
        npc_encounter_log: view
            .npc_encounter_log
            .into_iter()
            .map(|(k, v)| (k, serde_json::to_value(v).unwrap_or_default()))
            .collect(),
        narration_history_tail: view
            .narration_history_tail
            .into_iter()
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .collect(),
        narration_history_length: view.narration_history_length,
        dynamic_rooms: view.dynamic_rooms,
        dynamic_room_count: view.dynamic_room_count,
        last_error: view.last_error,
        quantifier_confidence: view.quantifier_confidence,
        backend_name: view.backend_name,
        model_name: view.model_name,
    };

    Ok(Json(response))
}
