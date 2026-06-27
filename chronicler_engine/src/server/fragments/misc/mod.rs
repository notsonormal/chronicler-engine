//! [DOC: docs/system/dashboard.md]
//! Miscellaneous fragment utilities (re-exports from submodules)

pub mod game_control;
pub mod swipe;
pub mod text_check;

pub use text_check::*;
pub use swipe::*;
pub use game_control::*;

#[cfg(test)]
mod misc_tests;
