//! [DOC: docs/system/dashboard.md]
//! Fragment endpoints

use axum::{extract::State, response::Html};

use crate::application::query_handlers;
use crate::adapters::driving::http::AppState;
use crate::application::context::OpContext;

use super::renderers::{
    render_action_area, render_character_headshots, render_header, render_llm_messages,
    render_story_log, render_visual_sidebar,
};

fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Html<String>
where
    F: FnOnce(&AppState) -> crate::error::Result<String>,
{
    match render(state) {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("{name} failed: {e}");
            Html(super::renderers::render_error(&e.to_string()))
        }
    }
}

pub async fn header_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_header, "header_fragment")
}

pub async fn story_log_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_story_log, "story_log_fragment")
}

pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_visual_sidebar, "visual_sidebar_fragment")
}

pub async fn action_area_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_action_area, "action_area_fragment")
}

pub async fn character_headshots_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(
        &state,
        render_character_headshots,
        "character_headshots_fragment",
    )
}

pub async fn llm_messages_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, render_llm_messages, "llm_messages_fragment")
}

pub async fn status_ready_handler(State(_state): State<AppState>) -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    let ctx = match crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game(&state) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("generating_status_handler: load_op_context failed: {e}");
            return Html("idle".to_string());
        }
    };
    tracing::debug!("generating_status_handler: called");
    let game_state = crate::application::context::load_expecting_valid_state(&ctx);
    let (status, phase) = match game_state {
        Ok(gs) => {
            tracing::debug!(
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
            tracing::error!("generating_status_handler: failed to load state: {e}");
            Default::default()
        }
    };
    let is_gen = status.is_generating();
    tracing::debug!(
        "generating_status_handler: is_generating={is_gen}, status={status:?}, phase={phase:?}",
    );
    tracing::debug!(
        "SERVER TRACE: status.is_generating()={is_gen}, status={status:?}, phase={phase:?}",
    );
    if let Some(err) = status.error_message() {
        tracing::debug!("SERVER TRACE: returning error span");
        Html(format!("<span class=\"status error\">Error: {err}</span>"))
    } else if is_gen {
        tracing::debug!(
            "SERVER TRACE: returning phase str: {}",
            phase.as_endpoint_str()
        );
        Html(phase.as_endpoint_str().to_string())
    } else {
        tracing::debug!("SERVER TRACE: returning idle");
        Html("idle".to_string())
    }
}

pub async fn reset_generating_handler(State(state): State<AppState>) -> Html<String> {
    match query_handlers::reset_generating_status(&state.application_service) {
        Ok(()) => Html("reset".to_string()),
        Err(e) => {
            tracing::error!("reset_generating_handler: {e}");
            Html("failed".to_string())
        }
    }
}
