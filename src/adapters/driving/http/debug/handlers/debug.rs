//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Debug utilities and endpoints

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::adapters::driving::http::AppState;

#[derive(Serialize)]
pub struct DebugBackendResponse {
    pub backend_name: String,
    pub model_name: String,
}

pub async fn debug_state_handler(
    State(state): State<AppState>,
) -> Result<Json<crate::application::DebugStateView>, StatusCode> {
    state
        .game_view_query
        .get_debug_state()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn debug_is_generating_handler(State(state): State<AppState>) -> String {
    // The generation registry is the live truth; the persisted status is a
    // record that can disagree with it.
    let game_id = state.game_catalogue.current_game_id();
    state.generation_gate.is_busy(game_id).to_string()
}

pub async fn debug_backend_handler(State(state): State<AppState>) -> Json<DebugBackendResponse> {
    // arch-lint: debug-direct — intentional exemption, see the hexagonal architecture docs.
    let (name, model) = state.pipeline.backend_info();
    Json(DebugBackendResponse {
        backend_name: name,
        model_name: model,
    })
}
