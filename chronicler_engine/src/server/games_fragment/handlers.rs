//! [DOC: docs/system/dashboard.md]
//! Games fragment handlers

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::server::AppState;

use crate::server::fragments::renderers::{
    app_err_to_response, app_err_to_tuple, internal_error, ok, ok_refresh, render_error,
    service_unavailable_generating,
};
use crate::server::games_fragment::template::{GameRowView, GamesPanelTemplate};

pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let ctx = match state.as_game_service_context() {
        Ok(c) => c,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };
    let games = match state.application_service.list_games(ctx.clone()) {
        Ok(g) => g,
        Err(e) => return internal_error(e.to_string()),
    };

    let active_id = state.application_service.current_game_id(ctx);

    let active_game = games
        .iter()
        .find(|g| g.id == active_id)
        .map(|g| GameRowView {
            id: g.id,
            name: g.name.clone(),
        });

    let saved_games: Vec<GameRowView> = games
        .into_iter()
        .filter(|g| g.id != active_id)
        .map(|g| GameRowView {
            id: g.id,
            name: g.name.clone(),
        })
        .collect();

    let template = GamesPanelTemplate {
        active_game,
        saved_games,
    };

    ok(template.render().unwrap_or_default())
}

pub async fn create_game_handler(State(state): State<AppState>) -> Response<axum::body::Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return service_unavailable_generating();
    }

    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };

    match state.application_service.create_game(ctx) {
        Ok(_) => ok_refresh(),
        Err(e) => app_err_to_response(e),
    }
}

pub async fn switch_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response<axum::body::Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return service_unavailable_generating();
    }

    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };
    match state.application_service.switch_game(ctx, id) {
        Ok(()) => ok_refresh(),
        Err(e) => app_err_to_response(e),
    }
}

pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> (StatusCode, String) {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return app_err_to_tuple(ApplicationError::ConcurrentGeneration);
    }

    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load context: {e}"),
            );
        }
    };
    match state.application_service.delete_game(ctx, id) {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(ApplicationError::Validation(msg)) => (StatusCode::BAD_REQUEST, render_error(&msg)),
        Err(ApplicationError::ConcurrentGeneration) => {
            app_err_to_tuple(ApplicationError::ConcurrentGeneration)
        }
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
