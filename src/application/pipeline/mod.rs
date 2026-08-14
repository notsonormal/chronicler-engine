//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action pipeline for processing game actions

pub mod action;
pub mod core;
pub mod phase_error;
pub mod phases;
pub mod retrigger;
pub mod retry;
pub mod spawn;

#[cfg(test)]
mod action_tests;
#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod retrigger_tests;
#[cfg(test)]
mod retry_tests;

pub use core::ActionPipeline;
pub use phase_error::PhaseError;
