//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Autoplay start handler

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Response},
};

use crate::adapters::driving::http::action::handlers::THINKING_STATUS;
use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::utils::response::ok;
use crate::adapters::driving::http::AppState;
use crate::application::autoplay::driver::{AutoplayRun, DEFAULT_BUDGET};
use crate::application::generation::gate::Refusal;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AutoplayForm {
    /// The budget in cycles; an absent or blank field starts a run of the default
    /// budget.
    #[serde(default)]
    pub cycles: String,
}

fn budget_of(form: &AutoplayForm) -> Result<u32, Refusal> {
    let cycles = form.cycles.trim();
    if cycles.is_empty() {
        return Ok(DEFAULT_BUDGET);
    }
    cycles
        .parse()
        .map_err(|_| Refusal::BadRequest("The budget must be a whole number of cycles".to_string()))
}

pub async fn start_autoplay_handler(
    State(state): State<AppState>,
    CurrentGame(game_id): CurrentGame,
    Form(form): Form<AutoplayForm>,
) -> Response {
    let budget = match budget_of(&form) {
        Ok(budget) => budget,
        Err(refusal) => return refusal.into_response(),
    };
    match AutoplayRun::start(&state.game_writer, state.job_deps.clone(), game_id, budget) {
        Ok(()) => ok(THINKING_STATUS),
        Err(refusal) => refusal.into_response(),
    }
}
