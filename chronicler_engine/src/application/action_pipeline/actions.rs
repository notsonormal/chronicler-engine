//! [DOC: docs/system/game_flow.md]
//! Action enum and action processing types

use tracing::instrument;
use crate::application::action_pipeline::phase_error::PhaseError;
use crate::application::application_service::DefaultApplicationService;

#[instrument(skip(app), fields(input_length))]
pub fn execute_action_impl(app: &DefaultApplicationService, input: String) {
    let mut state = app.load_or_fresh();
    state.narrative.last_trigger = None;
    let pipeline = app.game_service().pipeline();
    if let Err(PhaseError::Cancelled) = pipeline.run_from_input(app, state, input) {
        tracing::debug!("Pipeline cancelled");
    }
}
