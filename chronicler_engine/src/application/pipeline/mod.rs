//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Action pipeline for processing game actions

pub mod phase_error;
pub mod phases;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod spawn;

#[cfg(test)]
mod pipeline_tests;
#[cfg(test)]
mod retry_tests;

pub use phase_error::PhaseError;
pub use pipeline::ActionPipeline;
