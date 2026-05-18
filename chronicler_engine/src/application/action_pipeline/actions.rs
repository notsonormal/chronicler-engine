use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend,
};
use crate::application::context::{GameServiceContext, load_state};

/// [DOC: docs/architecture/system.md]
pub fn execute_action_impl<B: ActionPipelineBackend>(
    backend: &B,
    ctx: GameServiceContext,
    input: String,
    _player_name: String,
) {
    let mut state = load_state(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = ActionPipeline::new(backend, &ctx);
    match pipeline.run_from_input(state, input) {
        ActionOutcome::Completed => {}
        ActionOutcome::Error { message } => {
            log::error!("Action failed: {message}");
        }
        ActionOutcome::Cancelled => {}
    }
}
