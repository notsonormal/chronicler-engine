//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment endpoints

use axum::body::Body;
use axum::{extract::State, response::Html, response::Response};

use crate::adapters::driving::http::utils::error::{
    error_disclosure, generation_error_summary, raw_error_detail,
};
use crate::adapters::driving::http::utils::fragment::render_fragment;
use crate::adapters::driving::http::AppState;

pub async fn header_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(&state, |s| s.render_header(), "header_fragment")
}

pub async fn story_log_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(&state, |s| s.render_story_log(), "story_log_fragment")
}

pub async fn visual_sidebar_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_visual_sidebar(),
        "visual_sidebar_fragment",
    )
}

pub async fn options_dock_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(&state, |s| s.render_options_dock(), "options_dock_fragment")
}

pub async fn character_headshots_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_character_headshots(),
        "character_headshots_fragment",
    )
}

pub async fn llm_messages_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(&state, |s| s.render_llm_messages(), "llm_messages_fragment")
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
    // The live registry, not the persisted record, answers "is something
    // generating?". A persisted `Generating` with no slot is a stale artifact
    // from a panic; the poll reports `idle` so the page unblocks. The
    // persisted phase still supplies the phase name while a slot is live.
    let game_id = state.game_catalogue.current_game_id();
    let is_gen = state.generation_gate.is_busy(game_id);
    tracing::debug!(
        "generating_status_handler: is_generating={is_gen}, status={status:?}, phase={phase:?}",
    );
    if let Some(err) = status.error_message() {
        Html(error_disclosure(
            "status-error-popover",
            &generation_error_summary(err),
            &raw_error_detail(err),
        ))
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
