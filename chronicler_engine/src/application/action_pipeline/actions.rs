//! [DOC: docs/system/game_flow.md]
//! Action enum and action processing types

use tracing::instrument;
use crate::application::action_pipeline::pipeline::ActionOutcome;
use crate::application::context::{OpContext, load_or_fresh};
use crate::application::game_service::GameService;

#[instrument(skip(service, ctx), fields(input_length))]
pub fn execute_action_impl(service: &GameService, ctx: OpContext, input: String) {
    let mut state = load_or_fresh(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = service.pipeline();
    if let Err(ActionOutcome::Cancelled) = pipeline.run_from_input(&ctx, state, input) {
        tracing::debug!("Pipeline cancelled");
    }
}
