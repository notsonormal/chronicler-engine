//! [DOC: docs/diataxis/reference/game_flow.md]
//! ActionPipeline type-split: the struct and all its inherent impls.

pub mod action;
pub mod core;
pub mod options;
pub mod retrigger;
pub mod retry;

pub use core::ActionPipeline;

#[cfg(test)]
mod action_tests;
#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod retrigger_tests;
#[cfg(test)]
mod retry_tests;
