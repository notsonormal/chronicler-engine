use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::model::settings::TextCheckMode;
use crate::model::state::LogType;
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::templates::TextCheckPreviewTemplate;
use askama::Template;

use super::renderers::{render_action_area, render_error};

#[derive(Deserialize, Serialize)]
pub struct ActionForm {
    pub command: String,
}

/// [DOC: docs/system/game_flow.md]
#[allow(clippy::expect_used)]
async fn process_action(state: &AppState, command: String) -> Response<Body> {
    if command.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter a command</span>",
            ))
            .expect("static response body is valid");
    }

    let mut game_state = match state.load_state() {
        Ok(s) => s,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::new(String::new()))
                .expect("static response body is valid");
        }
    };

    let player_name = game_state.player.sheet.name.clone();
    game_state.add_log(command.clone(), Some(player_name.clone()), LogType::Input);
    let turn_id = game_state.narrative.current_turn_id.clone();

    // Reject concurrent actions while generation is in flight.
    if state
        .is_generating
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status wait\">Still thinking...</span>",
            ))
            .expect("static response body is valid");
    }

    game_state.narrative.generation.status = crate::model::state::GenerationStatus::Generating;
    game_state.narrative.generation.phase = crate::model::state::GenerationPhase::Narrating;
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &game_state,
        turn_id.clone(),
        0,
    );
    if let Err(e) = state.snapshot_storage.save(&snapshot) {
        log::error!("Failed to save snapshot: {e}");
    }

    let ctx = state.as_game_service_context();
    let cmd = command;
    let pname = player_name;
    let game_service = state.game_service.clone();
    let token = state.current_cancel_token();

    if token.is_cancelled() {
        let mut gs = match state.load_state() {
            Ok(s) => s,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .body(Body::from(render_error("Server is shutting down")))
                    .expect("static response body is valid");
            }
        };
        gs.narrative.generation.status = crate::model::state::GenerationStatus::Idle;
        let shutdown_turn_id = gs.narrative.current_turn_id.clone();
        let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
            &gs,
            shutdown_turn_id,
            0,
        );
        let _ = state.snapshot_storage.save(&snapshot);
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from(render_error("Server is shutting down")))
            .expect("static response body is valid");
    }

    // [DOC: docs/architecture/invariants.md#INV-004]
    tokio::task::spawn_blocking(move || {
        let _guard = crate::server::fragments::GenerationGuard(Arc::clone(&ctx.is_generating));
        if token.is_cancelled() {
            return;
        }
        game_service.execute_action(ctx, cmd, pname);
    });

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            "<span class=\"status thinking\">Thinking...</span>",
        ))
        .expect("static response body is valid")
}

/// [DOC: docs/system/game_flow.md]
#[allow(clippy::expect_used)]
pub async fn action_handler(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    process_action(&state, command).await
}

/// [DOC: docs/system/game_flow.md]
#[allow(clippy::expect_used)]
pub async fn action_confirm_handler(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    if command.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter a command</span>",
            ))
            .expect("static response body is valid");
    }

    let action_response = process_action(&state, command).await;
    let status = action_response.status();

    let action_area_html = match render_action_area(&state) {
        Ok(html) => html,
        Err(e) => {
            log::error!("Failed to render action area: {e}");
            render_error(&e.to_string())
        }
    };

    Response::builder()
        .status(status)
        .body(Body::from(action_area_html))
        .expect("static response body is valid")
}

/// [DOC: docs/system/text_check.md]
#[allow(clippy::expect_used)]
pub async fn action_check_handler(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    if command.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter a command</span>",
            ))
            .expect("static response body is valid");
    }

    let settings = state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled || !settings.text_check.enable_auto_check
    {
        let mut response = process_action(&state, command).await;
        add_status_swap_headers(&mut response);
        return response;
    }

    let result = match check_player_input(
        &command,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Text check failed: {e}");
            let mut response = process_action(&state, command).await;
            add_status_swap_headers(&mut response);
            return response;
        }
    };

    match result {
        Some(check_result) => {
            let template = TextCheckPreviewTemplate::from_check_result(&check_result);
            match template.render() {
                Ok(html) => Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(html))
                    .expect("static response body is valid"),
                Err(e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(render_error(&format!("Template error: {e}"))))
                    .expect("static response body is valid"),
            }
        }
        None => {
            let mut response = process_action(&state, command).await;
            add_status_swap_headers(&mut response);
            response
        }
    }
}

#[allow(clippy::expect_used)]
fn add_status_swap_headers(response: &mut Response<Body>) {
    response.headers_mut().insert(
        "HX-Retarget",
        "#status-display"
            .parse()
            .expect("static header value is valid"),
    );
    response.headers_mut().insert(
        "HX-Reswap",
        "innerHTML".parse().expect("static header value is valid"),
    );
}
