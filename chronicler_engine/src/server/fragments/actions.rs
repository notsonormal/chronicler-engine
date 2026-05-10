use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::engine::logic::get_current_room;
use crate::engine::parser::parse_command;
use crate::model::settings::TextCheckMode;
use crate::model::state::{GameState, LogType};
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

    let action = parse_command(&command);
    let is_sync = matches!(
        action,
        crate::engine::action::Action::Look
            | crate::engine::action::Action::Inventory
            | crate::engine::action::Action::Quit
    );

    if is_sync {
        process_sync_action(&mut game_state, &action);
        game_state.narrative.generation.status = crate::model::state::GenerationStatus::Idle;
        let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
            &game_state,
            uuid::Uuid::new_v4().to_string(),
            0,
        );
        if let Err(e) = state.snapshot_storage.save(&snapshot) {
            log::error!("Failed to save snapshot: {e}");
        }
    } else {
        game_state.narrative.generation.status = crate::model::state::GenerationStatus::Generating;
        game_state.narrative.generation.phase = crate::model::state::GenerationPhase::Narrating;
        let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
            &game_state,
            uuid::Uuid::new_v4().to_string(),
            0,
        );
        if let Err(e) = state.snapshot_storage.save(&snapshot) {
            log::error!("Failed to save snapshot: {e}");
        }

        let ctx = state.as_game_service_context();
        let cmd = command;
        let pname = player_name;
        let game_service = state.game_service.clone();
        let token = state.cancel_token.clone();

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
            let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                &gs,
                uuid::Uuid::new_v4().to_string(),
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
            if token.is_cancelled() {
                return;
            }
            game_service.execute_action(ctx, cmd, pname);
        });
    }

    if is_sync {
        Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "sync-action-complete")
            .body(Body::from("<span class=\"status ready\">Ready</span>"))
            .expect("static response body is valid")
    } else {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status thinking\">Thinking...</span>",
            ))
            .expect("static response body is valid")
    }
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

    let mut builder = Response::builder().status(status);
    if let Some(hx_trigger) = action_response.headers().get("HX-Trigger") {
        builder = builder.header("HX-Trigger", hx_trigger.clone());
    }
    builder
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

fn process_sync_action(state: &mut GameState, action: &crate::engine::action::Action) {
    match action {
        crate::engine::action::Action::Look => {
            if let Ok(room) = get_current_room(state) {
                state.add_log(
                    room.description.clone(),
                    Some(room.name.clone()),
                    LogType::Narration,
                );
            }
        }
        crate::engine::action::Action::Inventory => {
            state.add_log(
                "Your inventory is empty.".to_string(),
                None,
                LogType::System,
            );
        }
        crate::engine::action::Action::Quit => {
            state.add_log("Goodbye!".to_string(), None, LogType::System);
        }
        _ => {}
    }
}
