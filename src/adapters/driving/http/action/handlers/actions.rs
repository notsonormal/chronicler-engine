//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Action fragment handlers

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::domain::model::generation::JobRequest;
use crate::application::pipeline::job::Job;
use crate::domain::model::action::Action;
use crate::domain::model::game::GameId;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::templates::TextCheckPreviewTemplate;

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::failed_answer;
use crate::adapters::driving::http::utils::response::ok;

#[derive(Deserialize, Serialize)]
pub struct ActionForm {
    pub command: String,
}

fn dispatch_action(state: &AppState, game_id: GameId, command: String) -> Response<Body> {
    let request = match Action::parse(&command) {
        Action::FreeAction(text) => JobRequest::Action(text),
        Action::Guide(guide) => JobRequest::Guide(guide),
        Action::Impersonate(direction) => JobRequest::Impersonate(direction),
        Action::Options => JobRequest::Options,
    };
    match Job::start(&state.game_writer, state.job_deps.clone(), game_id, request) {
        Ok(()) => ok(""),
        Err(not_accepted) => not_accepted.into_response(),
    }
}

pub async fn action_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    dispatch_action(&state, game_id, command)
}

pub async fn action_check_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();

    if Action::parse(&command).is_engine_command() {
        return dispatch_action(&state, game_id, command);
    }

    let settings = match state.settings() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to read settings for text check: {e}");
            return dispatch_action(&state, game_id, command);
        }
    };

    if !settings.text_check.effective_enable_auto_check() {
        return dispatch_action(&state, game_id, command);
    }

    let result = match state.text_check_service().check_player_input(
        &command,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Text check failed: {e}");
            return dispatch_action(&state, game_id, command);
        }
    };

    match result {
        Some(check_result) => {
            let template = TextCheckPreviewTemplate::from_check_result(&check_result);
            match template.render() {
                Ok(html) => ok(html),
                Err(e) => failed_answer(format!("Template error: {e}")),
            }
        }
        None => dispatch_action(&state, game_id, command),
    }
}
