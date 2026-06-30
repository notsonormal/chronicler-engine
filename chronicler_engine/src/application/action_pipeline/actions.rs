//! [DOC: docs/system/game_flow.md]
//! Action enum and action processing types

use std::sync::Arc;

use tracing::instrument;
use crate::application::action_pipeline::pipeline::{ActionOutcome, ActionPipeline};
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::application::game_service::GameService;

#[instrument(skip(service, ctx), fields(input_length))]
pub fn execute_action_impl(service: &GameService, ctx: GameServiceContext, input: String) {
    let mut state = load_or_fresh(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = ActionPipeline::new(
        Arc::clone(&service.prompt_assembler),
        Arc::clone(&service.llm_recorder),
        Arc::clone(&service.agent_registry),
    );
    if let Err(ActionOutcome::Cancelled) = pipeline.run_from_input(&ctx, state, input) {
        tracing::debug!("Pipeline cancelled");
    }
}
