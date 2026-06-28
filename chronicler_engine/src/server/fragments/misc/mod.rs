//! [DOC: docs/system/dashboard.md]
//! Miscellaneous fragment utilities (re-exports from submodules)

pub mod game_control;
pub mod retrigger;
pub mod retry;
pub mod swipe;
pub mod text_check;

pub use game_control::reset_handler;
pub use retry::retry_handler;
pub use retrigger::retrigger_handler;
pub use swipe::switch_swipe_handler;
pub use text_check::*;

#[cfg(test)]
mod retrigger_tests;
#[cfg(test)]
mod retry_tests;
#[cfg(test)]
mod swipe_tests;
