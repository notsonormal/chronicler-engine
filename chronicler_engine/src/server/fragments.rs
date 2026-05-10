use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
    response::{Html, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::engine::logic::{get_available_exits, get_current_room};
use crate::engine::parser::parse_command;
use crate::error::Result;
use crate::model::settings::TextCheckMode;
use crate::model::state::{GameState, LogType};
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, StoryLogTemplate, TextCheckPreviewTemplate,
    VisualSidebarTemplate,
};

const MAX_LOG_DISPLAY: usize = 50;

pub(crate) fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

fn render_header_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;
    let template = HeaderTemplate {
        room_name: room.name.clone(),
    };
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_header(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;
    render_header_unlocked(&state_guard)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;

    let entries: Vec<_> = state_guard
        .narrative
        .history
        .iter()
        .take(MAX_LOG_DISPLAY)
        .cloned()
        .collect();
    let template = StoryLogTemplate::new(&entries);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

fn render_visual_sidebar_unlocked(state: &GameState) -> Result<String> {
    let room = get_current_room(state)?;

    let image_path = room
        .image_path
        .clone()
        .or_else(|| state.world.default_room_image.clone());

    let resolve_headshot = |npc_id: &str| {
        let npc = state.npcs.get(npc_id)?;
        let image_path = npc.sheet.preferred_image()?.to_string();
        let name = npc.sheet.name.clone();
        Some((image_path, name))
    };

    let npc_data: Vec<(String, String)> = if !state.scene.npcs_in_area.is_empty() {
        state
            .scene
            .npcs_in_area
            .iter()
            .filter_map(|npc| resolve_headshot(&npc.id))
            .collect()
    } else {
        // Fallback to static room.npcs
        room.npcs
            .iter()
            .filter_map(|npc_id| resolve_headshot(npc_id))
            .collect()
    };

    let template = VisualSidebarTemplate::new(image_path, room.name.clone(), npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;
    render_visual_sidebar_unlocked(&state_guard)
}

/// [DOC: docs/system/game_flow.md]
pub fn render_action_area(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;

    let status = state_guard.narrative.generation.status.clone();
    let phase = state_guard.narrative.generation.phase.clone();
    let exits = get_available_exits(&state_guard);
    drop(state_guard);

    let template = ActionAreaTemplate::new(&status, &phase, &exits);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Html<String>
where
    F: FnOnce(&AppState) -> Result<String>,
{
    match render(state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("{name} failed: {e}");
            Html(render_error(&e.to_string()))
        }
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn header_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_header, "header_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn story_log_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_story_log, "story_log_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_visual_sidebar, "visual_sidebar_fragment")
}

/// [DOC: docs/system/game_flow.md]
pub async fn action_area_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_action_area, "action_area_fragment")
}

fn render_character_headshots(state: &AppState) -> Result<String> {
    use crate::server::templates::CharacterHeadshotsTemplate;
    use askama::Template;

    let state_guard = state.load_state()?;

    let npc_data: Vec<(String, String)> = state_guard
        .npcs
        .iter()
        .filter_map(|(_npc_id, npc)| {
            let image = npc.sheet.preferred_image()?;
            let name = npc.sheet.name.clone();
            Some((image.to_string(), name))
        })
        .collect();

    let template = CharacterHeadshotsTemplate::new(npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

/// [DOC: docs/system/game_flow.md]
pub async fn character_headshots_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(
        &state,
        render_character_headshots,
        "character_headshots_fragment",
    )
}

/// [DOC: docs/system/game_flow.md]
pub async fn hints_handler(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_action_hints, "hints_handler")
}

pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

/// [DOC: docs/system/game_flow.md]
pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    let (status, phase) = match state.load_state() {
        Ok(guard) => (
            guard.narrative.generation.status.clone(),
            guard.narrative.generation.phase.clone(),
        ),
        _ => Default::default(),
    };

    if let Some(err) = status.error_message() {
        Html(format!("<span class=\"status error\">Error: {err}</span>"))
    } else if status.is_generating() {
        Html(phase.as_endpoint_str().to_string())
    } else {
        Html("idle".to_string())
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn reset_generating_handler(State(state): State<AppState>) -> Html<String> {
    let result = match state.load_state() {
        Ok(mut guard) => {
            guard.narrative.generation.status = crate::model::state::GenerationStatus::Idle;
            let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                &guard,
                uuid::Uuid::new_v4().to_string(),
                0,
            );
            state.snapshot_storage.save(&snapshot).is_ok()
        }
        Err(_) => false,
    };

    if result {
        Html("reset".to_string())
    } else {
        Html("failed".to_string())
    }
}

fn render_action_hints(state: &AppState) -> Result<String> {
    let state_guard = state.load_state()?;

    let exits = get_available_exits(&state_guard);
    let available_actions = if exits.is_empty() {
        String::from(
            "<span class=\"action-hint\">[Look]</span> <span class=\"action-hint\">[Inventory]</span>",
        )
    } else {
        let exit_hints: String = exits
            .iter()
            .map(|e| format!("<span class=\"action-hint\">[{e}]</span>"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "<span class=\"action-hint\">[Look]</span> <span class=\"action-hint\">[Inventory]</span> {exit_hints}"
        )
    };

    Ok(available_actions)
}

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

/// [DOC: docs/system/text_check.md]
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

/// [DOC: docs/system/text_check.md]
#[allow(clippy::expect_used)]
pub async fn check_text_handler(
    State(_state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let text = form.command.trim().to_string();
    if text.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "<span class=\"status error\">Enter text to check</span>",
            ))
            .expect("static response body is valid");
    }

    let settings = _state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled {
        return Response::builder()
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
        Ok(None) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                "<span class=\"status ready\">No issues found</span>",
            ))
            .expect("static response body is valid"),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(render_error(&format!("Check failed: {e}"))))
            .expect("static response body is valid"),
    }
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

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

/// [DOC: docs/system/game_flow.md]
pub async fn edit_history_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> (StatusCode, String) {
    let result = match state.load_state() {
        Ok(mut guard) => {
            let result = guard.edit_log(id, form.text);
            if result.is_ok() {
                let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                    &guard,
                    uuid::Uuid::new_v4().to_string(),
                    0,
                );
                let _ = state.snapshot_storage.save(&snapshot);
            }
            result
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Edited</span>".to_string(),
        ),
        Err(e) => (StatusCode::NOT_FOUND, render_error(&e.to_string())),
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn delete_history_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> (StatusCode, String) {
    let result = match state.load_state() {
        Ok(mut guard) => {
            let result = guard.delete_log(id);
            if result.is_ok() {
                let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                    &guard,
                    uuid::Uuid::new_v4().to_string(),
                    0,
                );
                let _ = state.snapshot_storage.save(&snapshot);
            }
            result
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(e) => (StatusCode::NOT_FOUND, render_error(&e.to_string())),
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
