//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! History fragment handlers

use axum::{
    extract::{Form, Path, State},
    response::Response,
};

use crate::application::errors::ApplicationError;
use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::utils::response::ok;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

pub async fn edit_history_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> Result<Response, ApplicationError> {
    state.persistence_gate.edit_history(id, form.text)?;
    Ok(ok("<span class=\"status ready\">Edited</span>"))
}

pub async fn delete_history_handler(
    State(state): State<AppState>,
) -> Result<Response, ApplicationError> {
    state.persistence_gate.delete_last()?;
    Ok(ok(""))
}
