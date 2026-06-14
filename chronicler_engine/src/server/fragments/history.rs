//! [DOC: docs/system/dashboard.md]
//! History fragment handlers

use axum::{extract::Form, extract::State, http::StatusCode};

use crate::application::application_service::ApplicationError;
use crate::server::AppState;

use super::renderers::render_error;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

pub async fn edit_history_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> (StatusCode, String) {
    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load context: {e}"),
            );
        }
    };
    match state.application_service.edit_history(ctx, id, form.text) {
        Ok(()) => (
            StatusCode::OK,
            "<span class=\"status ready\">Edited</span>".to_string(),
        ),
        Err(e) => (StatusCode::NOT_FOUND, render_error(&e.to_string())),
    }
}

pub async fn delete_history_handler(State(state): State<AppState>) -> (StatusCode, String) {
    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load context: {e}"),
            );
        }
    };
    match state.application_service.delete_last(ctx) {
        Ok(()) => (StatusCode::OK, String::new()),
        Err(ApplicationError::Validation(msg)) => (StatusCode::BAD_REQUEST, render_error(&msg)),
        Err(e) => (StatusCode::BAD_REQUEST, render_error(&e.to_string())),
    }
}
