//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment endpoints

use axum::body::Body;
use axum::{extract::State, response::Html, response::Response};

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::{
    error_disclosure, generation_failure_summary, raw_error_detail,
};
use crate::adapters::driving::http::utils::fragment::render_fragment;
use crate::adapters::driving::http::AppState;

pub async fn header_fragment(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    render_fragment(&state, |s| s.render_header(game_id), "header_fragment")
}

pub async fn story_log_fragment(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_story_log(game_id),
        "story_log_fragment",
    )
}

pub async fn visual_sidebar_fragment(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_visual_sidebar(game_id),
        "visual_sidebar_fragment",
    )
}

pub async fn options_dock_fragment(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_options_dock(game_id),
        "options_dock_fragment",
    )
}

pub async fn character_headshots_fragment(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_character_headshots(game_id),
        "character_headshots_fragment",
    )
}

pub async fn llm_messages_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(&state, |s| s.render_llm_messages(), "llm_messages_fragment")
}

pub async fn generating_status_handler(
    State(state): State<AppState>,
    current: Option<CurrentGame>,
) -> Html<String> {
    tracing::debug!("generating_status_handler: called");
    let game_id = current.map(|CurrentGame(game_id)| game_id);
    let (status, phase) = match game_id {
        Some(game_id) => match state.message_service.load_expecting_valid_state(game_id) {
            Ok(gs) => {
                tracing::debug!(
                    "generating_status_handler: loaded status={:?}, step={:?}",
                    gs.narrative.input_buffer.status,
                    gs.narrative.input_buffer.step
                );
                (
                    gs.narrative.input_buffer.status.clone(),
                    gs.narrative.input_buffer.step.clone(),
                )
            }
            Err(e) => {
                tracing::error!("generating_status_handler: failed to load state: {e}");
                Default::default()
            }
        },
        // No Game to read a persisted status from: the live registry has no
        // slot either, so the poll answers idle.
        None => Default::default(),
    };
    // The live registry, not the persisted record, answers "is something
    // generating?". A persisted `Generating` with no slot is a stale artifact
    // from a panic; the poll reports `idle` so the page unblocks. The
    // persisted phase still supplies the phase name while a slot is live.
    let is_gen = game_id.is_some_and(|game_id| state.generation_gate.is_busy(game_id));
    tracing::debug!(
        "generating_status_handler: is_generating={is_gen}, status={status:?}, phase={phase:?}",
    );
    if let Some(failure) = status.failure() {
        Html(error_disclosure(
            "status-error-popover",
            generation_failure_summary(failure.kind),
            &raw_error_detail(&failure.raw),
        ))
    } else if is_gen {
        Html(phase.as_endpoint_str().to_string())
    } else {
        Html("idle".to_string())
    }
}

pub async fn reset_generating_handler(
    State(state): State<AppState>,
    current: Option<CurrentGame>,
) -> Html<String> {
    let Some(CurrentGame(game_id)) = current else {
        return Html("failed".to_string());
    };
    let _ = state
        .generation_gate
        .release_generation_slot_for_game(game_id);
    match state.pipeline.reset_persisted_status(game_id) {
        Ok(()) => Html("reset".to_string()),
        Err(e) => {
            tracing::error!("reset_generating_handler: {e}");
            Html("failed".to_string())
        }
    }
}
