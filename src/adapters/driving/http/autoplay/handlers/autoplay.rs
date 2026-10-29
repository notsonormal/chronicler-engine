//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Autoplay start handler

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Response},
};

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::response::ok;
use crate::adapters::driving::http::AppState;
use crate::application::autoplay::budget::Budget;
use crate::application::autoplay::driver::AutoplayRun;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AutoplayForm {
    #[serde(default)]
    pub cycles: String,
}

pub async fn start_autoplay_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<AutoplayForm>,
) -> Response {
    let budget = match Budget::parse(&form.cycles) {
        Ok(budget) => budget,
        Err(not_accepted) => return not_accepted.into_response(),
    };
    match AutoplayRun::start(&state.game_writer, state.job_deps.clone(), game_id, budget) {
        Ok(()) => ok(""),
        Err(not_accepted) => not_accepted.into_response(),
    }
}
