//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
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
    state
        .game_view_query
        .get_generating_status()
        .map(|(status, _)| status.is_generating().to_string())
        .unwrap_or_else(|_| "false".to_string())
}

pub async fn debug_backend_handler(State(state): State<AppState>) -> Json<DebugBackendResponse> {
    // arch-lint: debug-direct — intentional exemption, see ADR-027 §3.2
    let (name, model) = state.pipeline.backend_info();
    Json(DebugBackendResponse {
        backend_name: name.to_string(),
        model_name: model.to_string(),
    })
}
