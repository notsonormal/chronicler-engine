//! [DOC: docs/system/game_flow.md]
//! Action enum and action processing types

use tracing::instrument;
use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend,
};
use crate::application::context::{GameServiceContext, load_or_fresh};

#[instrument(skip(backend, ctx), fields(player_name, input_length))]
pub fn execute_action_impl<B: ActionPipelineBackend>(
    backend: &B,
    ctx: GameServiceContext,
    input: String,
    _player_name: String,
) {
    let mut state = load_or_fresh(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = ActionPipeline::new(backend, &ctx);
    if let Err(ActionOutcome::Cancelled) = pipeline.run_from_input(state, input) {
        tracing::debug!("Pipeline cancelled");
    }
}
