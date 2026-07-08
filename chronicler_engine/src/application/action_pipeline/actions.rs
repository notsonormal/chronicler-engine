//! [DOC: docs/system/game_flow.md]
//! Action enum and action processing types

use tracing::instrument;
use crate::application::action_pipeline::pipeline::ActionOutcome;
use crate::application::application_service::DefaultApplicationService;

#[instrument(skip(app), fields(input_length))]
pub fn execute_action_impl(app: &DefaultApplicationService, input: String) {
    let Ok(mut state) = app.load_or_fresh() else {
        tracing::error!("execute_action: load_or_fresh failed");
        return;
    };
    state.narrative.last_trigger = None;
    let pipeline = app.game_service().pipeline();
    if let Err(ActionOutcome::Cancelled) = pipeline.run_from_input(app, state, input) {
        tracing::debug!("Pipeline cancelled");
    }
}