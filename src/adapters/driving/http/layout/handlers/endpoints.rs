//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment endpoints

use axum::body::Body;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    Json,
};

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::{
    action_failure_response, action_not_found_response, error_disclosure, raw_error_detail,
};
use crate::adapters::driving::http::utils::fragment::render_fragment;
use crate::adapters::driving::http::utils::response::html_escape;
use crate::adapters::driving::http::AppState;
use crate::application::games::view_query::Status;
use crate::application::generation::gate::Busy;
use crate::domain::model::change::ChangeVersions;
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};

pub async fn header_fragment(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response<Body> {
    render_fragment(&state, |s| s.render_header(game_id), "header_fragment")
}

pub async fn failure_banner_fragment(State(state): State<AppState>) -> Response<Body> {
    render_fragment(
        &state,
        |s| s.render_failure_banner(),
        "failure_banner_fragment",
    )
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

/// The dialog opens only on a 2xx answer: a missing record answers a 404 and a failed
/// read a 500, and the Open control reports either in the row's own slot.
pub async fn llm_message_record_fragment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response<Body> {
    match state.render_llm_message_record(id) {
        Ok(Some(html)) => Html(html).into_response(),
        Ok(None) => action_not_found_response("LLM message not found"),
        Err(e) => action_failure_response(format!("Load failed: {e}")),
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

pub async fn busy_status_handler(
    State(state): State<AppState>,
    current: Option<CurrentGame>,
) -> Html<String> {
    let Some(CurrentGame(game_id)) = current else {
        return Html(idle_answer());
    };
    match state.game_view_query.status(game_id) {
        Ok(Status::Busy(busy)) => Html(busy_answer(&busy)),
        Ok(Status::Failed(failure)) => Html(failure_answer(&failure)),
        Ok(Status::Idle) => Html(idle_answer()),
        Err(e) => {
            tracing::error!("busy_status_handler: {e}");
            Html(failure_answer(&GenerationFailure::unreadable(
                Record::Game,
                &e,
            )))
        }
    }
}

pub fn idle_answer() -> String {
    status_element("idle", "ready", "Ready")
}

/// The status display's answer: one element. `data-status` holds the machine
/// state that the action area reads, and the label is the text the display shows.
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
    current: Option<CurrentGame>,
) -> Html<String> {
    let Some(CurrentGame(game_id)) = current else {
        return Html("failed".to_string());
    };
    match state.game_writer.stop(game_id) {
        Ok(()) => Html("reset".to_string()),
        Err(e) => {
            tracing::error!("reset_busy_handler: {e}");
            Html("failed".to_string())
        }
    }
}
