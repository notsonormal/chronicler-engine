//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! History fragment handlers

use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Response},
};

use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::response::ok;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

pub async fn edit_history_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Path(id): Path<u64>,
    Form(form): Form<EditHistoryForm>,
) -> Response {
    match state.game_writer.edit(game_id, id, &form.text) {
        Ok(()) => ok(""),
        Err(refusal) => refusal.into_response(),
    }
}

pub async fn delete_history_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
) -> Response {
    match state.game_writer.delete_last(game_id) {
        Ok(()) => ok(""),
        Err(refusal) => refusal.into_response(),
    }
}
