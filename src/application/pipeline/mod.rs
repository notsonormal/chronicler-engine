//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action pipeline for processing game actions

pub mod action_pipeline;
pub mod phase_error;
pub mod pipeline_run;
pub mod spawn;

pub use action_pipeline::ActionPipeline;
pub use phase_error::PhaseError;
