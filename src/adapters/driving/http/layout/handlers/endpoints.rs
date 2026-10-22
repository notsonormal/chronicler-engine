//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment endpoints

use axum::body::Body;
use axum::{extract::State, response::Html, response::Response};

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::error::{error_disclosure, raw_error_detail};
use crate::adapters::driving::http::utils::fragment::render_fragment;
use crate::adapters::driving::http::AppState;
use crate::application::games::view_query::Status;
use crate::application::generation::gate::Busy;
use crate::domain::model::state::generation_status::{GenerationFailure, Record};

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
    let Some(CurrentGame(game_id)) = current else {
        return Html("idle".to_string());
    };
    match state.game_view_query.status(game_id) {
        Ok(Status::Busy(busy)) => Html(busy_answer(&busy)),
        Ok(Status::Failed(failure)) => Html(failure_disclosure(&failure)),
        Ok(Status::Idle) => Html("idle".to_string()),
        Err(e) => {
            tracing::error!("generating_status_handler: {e}");
            Html(failure_disclosure(&GenerationFailure::unreadable(
                Record::Game,
                &e,
            )))
        }
    }
}

fn busy_answer(busy: &Busy) -> String {
    match busy {
        Busy::Job(step) => step.as_endpoint_str().to_string(),
        Busy::Autoplay { progress, step } => {
            let mut answer = format!("autoplay {}/{}", progress.cycle(), progress.budget);
            if let Some(step) = step {
                answer.push(' ');
                answer.push_str(step.as_endpoint_str());
            }
            answer
        }
    }
}

fn failure_disclosure(failure: &GenerationFailure) -> String {
    error_disclosure(
        "status-error-popover",
        &failure.line(),
        &raw_error_detail(&failure.raw),
    )
}

pub async fn reset_generating_handler(
    State(state): State<AppState>,
    current: Option<CurrentGame>,
) -> Html<String> {
    let Some(CurrentGame(game_id)) = current else {
        return Html("failed".to_string());
    };
    match state.game_writer.stop(game_id) {
        Ok(()) => Html("reset".to_string()),
        Err(e) => {
            tracing::error!("reset_generating_handler: {e}");
            Html("failed".to_string())
        }
    }
}
