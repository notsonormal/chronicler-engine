use axum::{extract::State, response::Html};

use crate::server::AppState;

use super::renderers::{
    render_action_area, render_action_hints, render_character_headshots, render_header,
    render_llm_messages, render_story_log, render_visual_sidebar,
};

fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Html<String>
where
    F: FnOnce(&AppState) -> crate::error::Result<String>,
{
    match render(state) {
        Ok(html) => Html(html),
        Err(e) => {
            log::error!("{name} failed: {e}");
            Html(super::renderers::render_error(&e.to_string()))
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

pub async fn llm_messages_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_llm_messages, "llm_messages_fragment")
}

pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

/// [DOC: docs/system/game_flow.md]
pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    log::debug!("generating_status_handler: called");
    let ctx = state.as_game_service_context();
    // Load state fresh from storage
    let game_state = crate::application::context::try_load_state(&ctx);
    let (status, phase) = match game_state {
        Ok(gs) => {
            log::debug!(
                "generating_status_handler: loaded status={:?}, phase={:?}",
                gs.narrative.input_buffer.status,
                gs.narrative.input_buffer.phase
            );
            (
                gs.narrative.input_buffer.status.clone(),
                gs.narrative.input_buffer.phase.clone(),
            )
        }
        Err(e) => {
            log::error!("generating_status_handler: failed to load state: {e}");
            Default::default()
        }
    };
    let is_gen = status.is_generating();
    log::debug!(
        "generating_status_handler: is_generating={is_gen}, status={status:?}, phase={phase:?}",
    );
    log::info!("SERVER TRACE: status.is_generating()={is_gen}, status={status:?}, phase={phase:?}",);
    if let Some(err) = status.error_message() {
        log::info!("SERVER TRACE: returning error span");
        Html(format!("<span class=\"status error\">Error: {err}</span>"))
    } else if is_gen {
        log::info!(
            "SERVER TRACE: returning phase str: {}",
            phase.as_endpoint_str()
        );
        Html(phase.as_endpoint_str().to_string())
    } else {
        log::info!("SERVER TRACE: returning idle");
        Html("idle".to_string())
    }
}

/// [DOC: docs/system/game_flow.md]
pub async fn reset_generating_handler(State(state): State<AppState>) -> Html<String> {
    match state
        .application_service
        .reset_generating_status(state.as_game_service_context())
    {
        Ok(()) => Html("reset".to_string()),
        Err(_) => Html("failed".to_string()),
    }
}
