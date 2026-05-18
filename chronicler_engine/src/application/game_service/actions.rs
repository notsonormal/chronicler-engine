use super::action_pipeline::{ActionOutcome, ActionPipeline};
use super::context::GameServiceContext;
use super::helpers::load_state;
use super::service::DefaultGameService;

/// [DOC: docs/architecture/system.md]
pub fn execute_action_impl(
    service: &DefaultGameService,
    ctx: GameServiceContext,
    input: String,
    _player_name: String,
) {
    let _lock = match ctx.action_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut state = load_state(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = ActionPipeline::new(service, &ctx);
    match pipeline.run_from_input(state, input) {
        ActionOutcome::Completed => {}
        ActionOutcome::Error { message } => {
            log::error!("Action failed: {message}");
        }
        ActionOutcome::Cancelled => {}
    }
}
