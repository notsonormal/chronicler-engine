//! [DOC: docs/system/dashboard.md]
//! History fragment handlers

use axum::{extract::Form, extract::State, response::Response};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::server::AppState;

use super::renderers::{ctx_or_error, internal_error, ok, render_error};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

pub async fn edit_history_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> Response<axum::body::Body> {
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
    };
    match message_editing::edit_history(ctx, id, form.text) {
        Ok(()) => ok("<span class=\"status ready\">Edited</span>"),
        Err(e) => internal_error(render_error(&e.to_string())),
    }
}

pub async fn delete_history_handler(State(state): State<AppState>) -> Response<axum::body::Body> {
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
    };
    match message_editing::delete_last(ctx) {
        Ok(()) => ok(""),
        Err(ApplicationError::Validation(msg)) => internal_error(render_error(&msg)),
        Err(e) => internal_error(render_error(&e.to_string())),
    }
}
