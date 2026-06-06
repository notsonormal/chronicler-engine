//! [DOC: docs/system/dashboard.md]
//! Miscellaneous fragment utilities

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
};

use crate::application::application_service::ApplicationError;
use crate::model::settings::TextCheckMode;
use crate::narrative::text_check::check_player_input;
use crate::server::AppState;
use crate::server::templates::TextCheckPreviewTemplate;

use super::renderers::{
    app_err_to_tuple, bad_request, internal_error, ok, ok_refresh, service_unavailable,
    service_unavailable_generating,
};

#[allow(clippy::expect_used)]
pub async fn check_text_handler(
    State(state): State<AppState>,
    Form(form): Form<super::actions::ActionForm>,
) -> axum::response::Response<Body> {
    let text = form.command.trim().to_string();
    if text.is_empty() {
        return bad_request("<span class=\"status error\">Enter text to check</span>");
    }

    let settings = state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled {
        return ok("<span class=\"status ready\">Text check is disabled</span>");
    }

    match check_player_input(
        &text,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(Some(result)) => {
            let template = TextCheckPreviewTemplate::from_check_result(&result);
            match template.render() {
                Ok(html) => ok(html),
                Err(e) => internal_error(format!("Template error: {e}")),
            }
        }
        Ok(None) => ok("<span class=\"status ready\">No issues found</span>"),
        Err(e) => internal_error(format!("Check failed: {e}")),
    }
}

pub async fn retry_handler(State(state): State<AppState>) -> (StatusCode, String) {
    match state
        .application_service
        .retry(state.as_game_service_context())
    {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Retrying...</span>".to_string(),
        ),
        Err(e) => app_err_to_tuple(e),
    }
}

/// Requires `last_trigger` to be present and the last message to be a narration.
pub async fn retrigger_handler(State(state): State<AppState>) -> (StatusCode, String) {
    match state
        .application_service
        .retrigger(state.as_game_service_context())
    {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Retriggering...</span>".to_string(),
        ),
        Err(e) => app_err_to_tuple(e),
    }
}

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    axum::extract::Path((message_id, swipe_index)): axum::extract::Path<(u64, usize)>,
) -> axum::response::Response<Body> {
    match state.application_service.switch_swipe(
        state.as_game_service_context(),
        message_id,
        swipe_index,
    ) {
        Ok(()) => match super::renderers::render_story_log(&state) {
            Ok(html) => ok(html),
            Err(e) => internal_error(format!("Failed to render story log: {e}")),
        },
        Err(ApplicationError::Validation(msg)) => {
            bad_request(format!("<span class=\"status error\">{msg}</span>"))
        }
        Err(ApplicationError::ConcurrentGeneration) => service_unavailable_generating(),
        Err(ApplicationError::ShuttingDown) => {
            service_unavailable("<span class=\"status error\">Server is shutting down</span>")
        }
        Err(e) => internal_error(format!("Failed to switch swipe: {e}")),
    }
}

pub async fn reset_handler(State(state): State<AppState>) -> axum::response::Response<Body> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return service_unavailable_generating();
    }

    state.current_cancel_token().cancel();

    match state
        .application_service
        .reset(state.as_game_service_context())
    {
        Ok(()) => {
            state
                .is_generating
                .store(false, std::sync::atomic::Ordering::SeqCst);
            state.replace_cancel_token();
            ok_refresh()
        }
        Err(e) => internal_error(e.to_string()),
    }
}
