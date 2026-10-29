//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Chat window HTTP request handlers.

use axum::{
    body::Body,
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Response},
};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::{AnswerPolicy, answer_for_error};
use crate::adapters::driving::http::utils::handler_helpers::{LastMessageForm, names_no_message};
use crate::adapters::driving::http::utils::response::ok;
use crate::domain::model::generation::JobRequest;
use crate::application::pipeline::job::Job;
use crate::domain::model::game::GameId;

pub async fn index_handler() -> Html<String> {
    Html(include_str!("../../../../../../assets/index.html").to_string())
}

pub async fn reset_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    match state.game_catalogue.reset(game_id) {
        Ok(()) => ok(""),
        Err(e) => answer_for_error(e, None, AnswerPolicy::EngineText),
    }
}

pub async fn retrigger_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<LastMessageForm>,
) -> Response<Body> {
    match form.named() {
        Some(message) => start_job(&state, game_id, JobRequest::Retrigger(message)),
        None => names_no_message(),
    }
}

pub async fn retry_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<LastMessageForm>,
) -> Response<Body> {
    match form.named() {
        Some(message) => start_job(&state, game_id, JobRequest::Retry(message)),
        None => names_no_message(),
    }
}

fn start_job(state: &AppState, game_id: GameId, request: JobRequest) -> Response<Body> {
    match Job::start(&state.game_writer, state.job_deps.clone(), game_id, request) {
        Ok(()) => ok(""),
        Err(not_accepted) => not_accepted.into_response(),
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
        Ok(()) => ok(""),
        Err(not_accepted) => not_accepted.into_response(),
    }
}
