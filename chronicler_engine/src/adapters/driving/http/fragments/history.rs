//! [DOC: docs/system/dashboard.md]
//! History fragment handlers

use axum::{
    extract::{Form, Path, State},
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::adapters::driving::http::AppState;
use crate::error::EngineError;

use super::renderers::ok;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

pub async fn edit_history_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> Result<Response, ApplicationError> {
    let ctx = state
        .as_game_service_context()
        .map_err(|e| ApplicationError::Engine(EngineError::Render(e.to_string())))?;
    message_editing::edit_history(ctx, id, form.text)?;
    Ok(ok("<span class=\"status ready\">Edited</span>"))
}

pub async fn delete_history_handler(
    State(state): State<AppState>,
) -> Result<Response, ApplicationError> {
    let ctx = state
        .as_game_service_context()
        .map_err(|e| ApplicationError::Engine(EngineError::Render(e.to_string())))?;
    message_editing::delete_last(ctx)?;
    Ok(ok(""))
}
