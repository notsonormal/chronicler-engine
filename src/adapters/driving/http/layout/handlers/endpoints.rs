//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The status display, the changes answer, the LLM record dialog and the busy reset.

use axum::body::Body;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    Json,
};

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::{
    AnswerPolicy, answer_for_error, failed_answer, not_found_answer, error_disclosure,
    raw_error_detail,
};
use crate::adapters::driving::http::utils::response::{html_escape, ok};
use crate::adapters::driving::http::AppState;
use crate::application::games::view_query::Status;
use crate::application::generation::gate::Busy;
use crate::domain::model::change::ChangeVersions;
use crate::domain::model::game::GameId;
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};

/// The dialog opens only on a 2xx answer: a missing record answers a 404 and a failed
/// read a 500, and the notice reports either.
pub async fn llm_message_record_fragment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response<Body> {
    match state.render_llm_message_record(id) {
        Ok(Some(html)) => Html(html).into_response(),
        Ok(None) => not_found_answer("LLM message not found"),
        Err(e) => failed_answer(e),
    }
}

pub async fn changes_handler(
    State(state): State<AppState>,
    current: Option<CurrentGame>,
) -> Json<ChangeVersions> {
    Json(
        state
            .game_view_query
            .change_versions(current.map(|CurrentGame(game_id)| game_id)),
    )
}

/// A Game whose status cannot be read shows its failure line in the display, not an
/// error answer.
pub fn render_status_display(state: &AppState, game: Option<GameId>) -> String {
    let Some(game_id) = game else {
        return idle_answer();
    };
    match state.game_view_query.status(game_id) {
        Ok(Status::Busy(busy)) => busy_answer(&busy),
        Ok(Status::Failed(failure)) => failure_answer(&failure),
        Ok(Status::Idle) => idle_answer(),
        Err(e) => {
            tracing::error!("render_status_display: {e}");
            failure_answer(&GenerationFailure::unreadable(Record::Game, &e))
        }
    }
}

pub fn idle_answer() -> String {
    status_element("idle", "ready", "Ready")
}

/// `data-status` holds the machine state that the action area reads.
fn status_element(state: &str, class: &str, label: &str) -> String {
    format!(
        "<span class=\"status {class}\" data-status=\"{state}\">{}</span>",
        html_escape(label)
    )
}

fn step_label(step: &Step) -> &'static str {
    match step {
        Step::Narrating => "Generating narration...",
        Step::Quantifying => "Quantifying...",
        Step::GeneratingEvent => "Generating event...",
        Step::Options => "Generating options...",
    }
}

pub fn busy_answer(busy: &Busy) -> String {
    match busy {
        Busy::Job(step) => status_element("thinking", "thinking", step_label(step)),
        Busy::Autoplay { progress, step } => {
            let mut label = format!("Autoplay: cycle {}/{}", progress.cycle(), progress.budget);
            if let Some(step) = step {
                label.push(' ');
                label.push_str(step_label(step));
            }
            status_element("autoplay", "thinking autoplay", &label)
        }
    }
}

pub fn failure_answer(failure: &GenerationFailure) -> String {
    let disclosure = error_disclosure(
        "status-error-popover",
        &failure.line(),
        &raw_error_detail(&failure.raw),
    );
    format!("<div data-status=\"failed\">{disclosure}</div>")
}

pub async fn reset_busy_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    match state.game_writer.stop(game_id) {
        Ok(()) => ok("reset"),
        Err(e) => answer_for_error(e.into(), None, AnswerPolicy::EngineText),
    }
}
