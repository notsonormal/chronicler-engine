//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Action fragment handlers

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::application::generation::gate::JobRequest;
use crate::application::pipeline::job::Job;
use crate::domain::model::action::Action;
use crate::domain::model::game::GameId;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::headers::add_status_swap_headers;
use crate::adapters::driving::http::templates::TextCheckPreviewTemplate;

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::response::{internal_error, ok};

/// `dashboard-action-area.js` keys the wait state on the `.status.wait` class in this
/// markup; the stub server answers a refused Action with it.
pub const BUSY_STATUS: &str = "<span class=\"status wait\">Still thinking...</span>";

#[derive(Deserialize, Serialize)]
pub struct ActionForm {
    pub command: String,
}

async fn dispatch_action(state: &AppState, game_id: GameId, command: String) -> Response<Body> {
    let request = match Action::parse(&command) {
        Action::FreeAction(text) => JobRequest::Action(text),
        Action::Guide(guide) => JobRequest::Guide(guide),
        Action::Impersonate(direction) => JobRequest::Impersonate(direction),
        Action::Options => JobRequest::Options,
    };
    match Job::start(&state.game_writer, state.job_deps.clone(), game_id, request) {
        Ok(()) => ok("<span class=\"status thinking\">Thinking...</span>"),
        Err(refusal) => refusal.into_response(),
    }
}

pub async fn action_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    dispatch_action(&state, game_id, command).await
}

#[allow(clippy::expect_used)]
pub async fn action_confirm_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();
    dispatch_with_status_headers(&state, game_id, command).await
}

async fn dispatch_with_status_headers(
    state: &AppState,
    game_id: GameId,
    command: String,
) -> Response<Body> {
    let mut response = dispatch_action(state, game_id, command).await;
    add_status_swap_headers(&mut response);
    response
}

pub async fn action_check_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<ActionForm>,
) -> Response<Body> {
    let command = form.command.trim().to_string();

    if Action::parse(&command).is_engine_command() {
        return dispatch_with_status_headers(&state, game_id, command).await;
    }

    let settings = match state.settings() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to read settings for text check: {e}");
            return dispatch_with_status_headers(&state, game_id, command).await;
        }
    };

    if !settings.text_check.effective_enable_auto_check() {
        return dispatch_with_status_headers(&state, game_id, command).await;
    }

    let result = match state.text_check_service().check_player_input(
        &command,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Text check failed: {e}");
            return dispatch_with_status_headers(&state, game_id, command).await;
        }
    };

    match result {
        Some(check_result) => {
            let template = TextCheckPreviewTemplate::from_check_result(&check_result);
            match template.render() {
                Ok(html) => ok(html),
                Err(e) => internal_error(render_error(&format!("Template error: {e}"))),
            }
        }
        None => dispatch_with_status_headers(&state, game_id, command).await,
    }
}
