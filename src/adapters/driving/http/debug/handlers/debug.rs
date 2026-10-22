//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Debug utilities and endpoints

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::current_game::CurrentGame;

#[derive(Serialize)]
pub struct DebugBackendResponse {
    pub backend_name: String,
    pub model_name: String,
}

pub async fn debug_state_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Result<Json<crate::application::DebugStateView>, StatusCode> {
    let mut view = state
        .game_view_query
        .get_debug_state(game_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // The Game's record holds no call, so the name and model come from the
    // provider the app itself runs on, as `debug_backend_handler` reports it.
    let (backend_name, model_name) = state.job_deps.narrator.backend_info();
    view.backend_name = Some(backend_name);
    view.model_name = Some(model_name);
    Ok(Json(view))
}

pub async fn debug_is_generating_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> String {
    // The holder is the live truth: a Game is busy while a Job or an Autoplay run holds it.
    state.game_view_query.is_busy(game_id).to_string()
}

pub async fn debug_backend_handler(State(state): State<AppState>) -> Json<DebugBackendResponse> {
    // arch-lint: debug-direct — intentional exemption, see the hexagonal architecture docs.
    let (name, model) = state.job_deps.narrator.backend_info();
    Json(DebugBackendResponse {
        backend_name: name,
        model_name: model,
    })
}
