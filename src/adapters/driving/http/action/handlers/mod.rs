//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Action route handlers.

mod actions;

pub use actions::{action_check_handler, action_confirm_handler, action_handler, ActionForm};

#[cfg(test)]
mod actions_tests;
