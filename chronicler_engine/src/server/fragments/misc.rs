use std::sync::Arc;

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
    response::Html,
};

use crate::model::settings::TextCheckMode;
use crate::model::state::GameState;
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::templates::TextCheckPreviewTemplate;

use super::renderers::{
    render_action_area, render_error, render_header, render_story_log, render_visual_sidebar,
};

/// [DOC: docs/system/text_check.md]
#[allow(clippy::expect_used)]
pub async fn check_text_handler(
    State(_state): State<AppState>,
    Form(form): Form<super::actions::ActionForm>,
) -> axum::response::Response<Body> {
    let text = form.command.trim().to_string();
    if text.is_empty() {
        return axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter text to check</span>",
            ))
            .expect("static response body is valid");
    }

    let settings = _state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled {
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status ready\">Text check is disabled</span>",
            ))
            .expect("static response body is valid");
    }

    match check_player_input(
        &text,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(Some(result)) => {
            let template = TextCheckPreviewTemplate::from_check_result(&result);
            match template.render() {
                Ok(html) => axum::response::Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(html))
                    .expect("static response body is valid"),
                Err(e) => axum::response::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(render_error(&format!("Template error: {e}"))))
                    .expect("static response body is valid"),
            }
        }
        Ok(None) => axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status ready\">No issues found</span>",
            ))
            .expect("static response body is valid"),
        Err(e) => axum::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(render_error(&format!("Check failed: {e}"))))
            .expect("static response body is valid"),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn retry_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let has_input = match state.load_state() {
        Ok(g) => g.get_last_input_text().is_some(),
        Err(_) => false,
    };
    if !has_input {
        return (StatusCode::BAD_REQUEST, render_error("No input to retry"));
    }

    let ctx = state.as_game_service_context();
    let game_service = state.game_service.clone();
    let token = state.cancel_token.clone();

    if token.is_cancelled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            render_error("Server is shutting down"),
        );
    }

    // [DOC: docs/architecture/invariants.md#INV-004]
    // Retry runs off the async thread so the HTTP handler returns immediately.
    tokio::task::spawn_blocking(move || {
        if token.is_cancelled() {
            return;
        }
        game_service.retry_last_response(ctx);
    });

    (
        StatusCode::OK,
        "<span class=\"status ready\">Retrying...</span>".to_string(),
    )
}

/// [DOC: docs/system/game_flow.md]
pub async fn reset_handler(State(state): State<AppState>) -> Html<String> {
    state.cancel_token.cancel();

    if let Err(e) = state.snapshot_storage.reset() {
        log::error!("Reset failed: {e}");
        return Html(render_error(&e.to_string()));
    }

    let initial_state = GameState::new(
        Arc::clone(&state.world),
        Arc::clone(&state.map),
        Arc::clone(&state.player),
        (*state.npcs).values().cloned().collect(),
        state.starting_room_id.clone(),
    );

    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &initial_state,
        "initial".to_string(),
        0,
    );
    let _ = state.snapshot_storage.save(&snapshot);

    let header = render_header(&state).unwrap_or_default();
    let story = render_story_log(&state).unwrap_or_default();
    let sidebar = render_visual_sidebar(&state).unwrap_or_default();
    let action = render_action_area(&state).unwrap_or_default();

    Html(format!("{header}{story}{sidebar}{action}"))
}
