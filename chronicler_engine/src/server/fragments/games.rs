use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use crate::model::game::generate_game_name;
use crate::server::AppState;

use super::renderers::{html_escape, render_error};

/// [DOC: docs/system/game_flow.md]
fn error_response(message: &str) -> Response<axum::body::Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(axum::body::Body::from(render_error(message)))
        .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")))
}

/// [DOC: docs/system/game_flow.md]
pub async fn list_games_fragment(State(state): State<AppState>) -> Response<axum::body::Body> {
    let games = match state.snapshot_storage.list_games() {
        Ok(g) => g,
        Err(e) => return error_response(&e.to_string()),
    };

    let active_id = state.snapshot_storage.current_game_id();
    let current_world = &state.world.name;

    let mut html = String::new();

    // Active game section
    let active_game = games
        .iter()
        .find(|g| g.id == active_id && &g.world_name == current_world);
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

    let world_games: Vec<_> = games
        .into_iter()
        .filter(|g| &g.world_name == current_world && g.id != active_id)
        .collect();

    if world_games.is_empty() {
        html.push_str("<div class=\"games-empty\">No saved games for this world.</div>");
    } else {
        for game in world_games {
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

    Response::builder()
        .status(StatusCode::OK)
        .body(axum::body::Body::from(html))
        .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")))
}

/// [DOC: docs/system/game_flow.md]
pub async fn create_game_handler(State(state): State<AppState>) -> Response<axum::body::Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(axum::body::Body::from(
                "<span class=\"status wait\">Generation in progress, please wait...</span>",
            ))
            .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
    }

    let world_name = state.world.name.clone();
    let new_id = match state.snapshot_storage.list_games().and_then(|games| {
        let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        let name = generate_game_name(&world_name, &existing_names);
        state.snapshot_storage.create_game(&world_name, &name)
    }) {
        Ok(id) => id,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(render_error(&e.to_string())))
                .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
        }
    };

    // Switch to the new game so we can initialize it with starting state.
    let old_id = state.snapshot_storage.current_game_id();
    state.snapshot_storage.set_game_id(new_id);
    state.message_storage.set_game_id(new_id);

    // Build and persist initial state (scenario logs + snapshot).
    let mut initial_state = super::misc::build_fresh_initial_state(&state);
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&initial_state);
    let snapshot_id = match state.snapshot_storage.save(&snapshot) {
        Ok(id) => id,
        Err(e) => {
            // Restore old game before returning error.
            state.snapshot_storage.set_game_id(old_id);
            state.message_storage.set_game_id(old_id);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(render_error(&e.to_string())))
                .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
        }
    };

    if let Some(msg) = initial_state.narrative.history.last_mut() {
        if msg.id == 0 {
            msg.snapshot_id = Some(snapshot_id);
            if let Err(e) = state.message_storage.insert_message(msg) {
                log::error!("Create game failed: could not persist message: {e}");
            }
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Refresh", "true")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")))
}

/// [DOC: docs/system/game_flow.md]
pub async fn switch_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response<axum::body::Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(axum::body::Body::from(
                "<span class=\"status wait\">Generation in progress, please wait...</span>",
            ))
            .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
    }

    match state.snapshot_storage.get_game(id) {
        Ok(Some(game)) => {
            if game.world_name != state.world.name {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from(render_error(
                        "Game belongs to a different world",
                    )))
                    .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
            }
        }
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(render_error("Game not found")))
                .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
        }
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(render_error(&e.to_string())))
                .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")));
        }
    }

    state.snapshot_storage.set_game_id(id);
    state.message_storage.set_game_id(id);

    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Refresh", "true")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| Response::new(axum::body::Body::from("Internal error")))
}

/// [DOC: docs/system/game_flow.md]
pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> (StatusCode, String) {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "<span class=\"status wait\">Generation in progress, please wait...</span>".to_string(),
        );
    }

    if id == state.snapshot_storage.current_game_id() {
        return (
            StatusCode::BAD_REQUEST,
            render_error("Cannot delete the active game"),
        );
    }

    match state.snapshot_storage.delete_game(id) {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
