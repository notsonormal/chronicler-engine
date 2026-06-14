//! [DOC: docs/system/dashboard.md]
//! Games fragment handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::server::AppState;

use super::renderers::{
    app_err_to_response, app_err_to_tuple, html_escape, internal_error, ok, ok_refresh,
    render_error, service_unavailable_generating,
};

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

    let mut html = String::new();

    // Active game section
    let active_game = games.iter().find(|g| g.id == active_id);
    html.push_str("<div class=\"save-load-panel\">");
    html.push_str("<div class=\"save-load-section\">");
    html.push_str("<h2>Active Game</h2>");
    if let Some(game) = active_game {
        let safe_name = html_escape(&game.name);
        html.push_str(&format!(
            r#"<div class="game-item active">
                <span class="game-name">{safe_name}</span>
                <span class="game-badge">Current</span>
            </div>"#
        ));
    } else {
        html.push_str(
            "<div class=\"game-item\"><span class=\"game-name\">No active game</span></div>",
        );
    }
    html.push_str("</div>");

    // All games list
    html.push_str("<div class=\"save-load-section\">");
    html.push_str("<h2>Saved Games</h2>");
    html.push_str("<div class=\"games-list\">");

    let saved_games: Vec<_> = games.into_iter().filter(|g| g.id != active_id).collect();

    if saved_games.is_empty() {
        html.push_str("<div class=\"games-empty\">No saved games.</div>");
    } else {
        for game in saved_games {
            let safe_name = html_escape(&game.name);
            html.push_str(&format!(
                r#"<div class="game-item" data-id="{}">
                    <span class="game-name">{}</span>
                    <div class="game-actions">
                        <button class="btn-switch" hx-post="/games/{}/switch" hx-swap="none">Switch</button>
                        <button class="btn-delete" hx-post="/games/{}/delete" hx-target="closest .game-item" hx-swap="outerHTML" hx-confirm="Delete this game? This cannot be undone.">Delete</button>
                    </div>
                </div>"#,
                game.id, safe_name, game.id, game.id
            ));
        }
    }

    html.push_str("</div>"); // games-list
    html.push_str("</div>"); // save-load-section

    // Actions row
    html.push_str("<div class=\"save-load-actions\">");
    html.push_str(
        r#"<button class="btn-new-game" hx-post="/games" hx-swap="none">New Game</button>"#,
    );
    html.push_str(r#"<button class="btn-reset" hx-post="/reset" hx-confirm="Are you sure you want to reset the current game? All progress will be lost." hx-swap="none">Reset Current Game</button>"#);
    html.push_str("</div>");

    html.push_str("</div>"); // save-load-panel

    ok(html)
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
