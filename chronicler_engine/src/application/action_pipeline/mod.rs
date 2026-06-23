//! [DOC: docs/system/game_flow.md]
//! Action pipeline for processing game actions

pub mod actions;
pub mod phases;
pub mod pipeline;
pub mod retry;

#[cfg(test)]
mod actions_tests;
#[cfg(test)]
mod pipeline_tests;
#[cfg(test)]
mod retry_tests;

pub use actions::execute_action_impl;
pub use pipeline::{ActionOutcome, ActionPipeline, ActionPipelineBackend};
pub use retry::retrigger_event_impl;
pub use retry::retry_last_response_impl;
