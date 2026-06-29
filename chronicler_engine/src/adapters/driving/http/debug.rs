//! [DOC: docs/system/dashboard.md]
//! Debug utilities and endpoints

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::application::query_handlers;
use crate::adapters::driving::http::AppState;

#[derive(Serialize)]
pub struct DebugBackendResponse {
    pub backend_name: String,
    pub model_name: String,
}

/// NOTE: dev-only diagnostic endpoint
pub async fn debug_state_handler(
    State(state): State<AppState>,
) -> Result<Json<crate::application::DebugStateView>, StatusCode> {
    let ctx = state.as_game_service_context().map_err(|e| {
        tracing::error!("State load failed during /debug/state request: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    query_handlers::get_debug_state(ctx)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn debug_is_generating_handler(State(state): State<AppState>) -> String {
    state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
        .to_string()
}

pub async fn debug_backend_handler(State(state): State<AppState>) -> Json<DebugBackendResponse> {
    let (name, model) = state.game_service.backend_info();
    Json(DebugBackendResponse {
        backend_name: name.to_string(),
        model_name: model.to_string(),
    })
}
