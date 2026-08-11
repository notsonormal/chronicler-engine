//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment endpoints

use axum::{extract::State, response::Html};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::fragment::render_fragment;

pub async fn header_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, |s| s.render_header(), "header_fragment")
}

pub async fn story_log_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, |s| s.render_story_log(), "story_log_fragment")
}

pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(
        &state,
        |s| s.render_visual_sidebar(),
        "visual_sidebar_fragment",
    )
}

pub async fn action_area_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, |s| s.render_action_area(), "action_area_fragment")
}

pub async fn character_headshots_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(
        &state,
        |s| s.render_character_headshots(),
        "character_headshots_fragment",
    )
}

pub async fn llm_messages_fragment(State(state): State<AppState>) -> Html<String> {
    render_fragment(&state, |s| s.render_llm_messages(), "llm_messages_fragment")
}

pub async fn status_ready_handler() -> Html<String> {
    Html("<span class=\"status ready\">Ready</span>".to_string())
}

pub async fn generating_status_handler(State(state): State<AppState>) -> Html<String> {
    tracing::debug!("generating_status_handler: called");
    let game_state = state.message_service.load_expecting_valid_state();
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
    if let Some(err) = status.error_message() {
        Html(format!("<span class=\"status error\">Error: {err}</span>"))
    } else if is_gen {
        Html(phase.as_endpoint_str().to_string())
    } else {
        Html("idle".to_string())
    }
}

pub async fn reset_generating_handler(State(state): State<AppState>) -> Html<String> {
    let game_id = state.game_catalogue.current_game_id();
    let _ = state
        .generation_gate
        .release_generation_slot_for_game(game_id);
    match state.pipeline.reset_persisted_status() {
        Ok(()) => Html("reset".to_string()),
        Err(e) => {
            tracing::error!("reset_generating_handler: {e}");
            Html("failed".to_string())
        }
    }
}
