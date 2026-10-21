//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Chat window HTTP request handlers.

use axum::{
    body::Body,
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::response::{internal_error, ok, ok_refresh};
use crate::application::errors::ApplicationError;
use crate::domain::model::generation::JobRequest;
use crate::application::pipeline::job::Job;
use crate::domain::model::game::GameId;

pub async fn index_handler() -> Html<String> {
    Html(include_str!("../../../../../../assets/index.html").to_string())
}

pub async fn reset_handler(
    State(state): State<AppState>,
) -> Result<axum::response::Response<Body>, ApplicationError> {
    match state.game_catalogue.reset() {
        Ok(()) => Ok(ok_refresh()),
        Err(e) => Ok(internal_error(e.to_string())),
    }
}

pub async fn retrigger_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    start_job(&state, game_id, JobRequest::Retrigger, "Retriggering...")
}

pub async fn retry_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    start_job(&state, game_id, JobRequest::Retry, "Retrying...")
}

fn start_job(
    state: &AppState,
    game_id: GameId,
    request: JobRequest,
    label: &str,
) -> Response<Body> {
    match Job::start(&state.game_writer, state.job_deps.clone(), game_id, request) {
        Ok(()) => ok(format!("<span class=\"status ready\">{label}</span>")),
        Err(refusal) => refusal.into_response(),
    }
}

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Path((message_id, swipe_index)): Path<(u64, usize)>,
) -> Response<Body> {
    match state
        .game_writer
        .switch_swipe(game_id, message_id, swipe_index)
    {
        Ok(()) => match state.render_story_log(game_id) {
            Ok(html) => ok(html),
            Err(e) => internal_error(render_error(&format!(
                "Failed to render the story log: {e}"
            ))),
        },
        Err(refusal) => refusal.into_response(),
    }
}
