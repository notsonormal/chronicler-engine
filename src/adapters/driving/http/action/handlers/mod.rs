//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Action route handlers.

mod actions;

pub use actions::{
    action_check_handler, action_confirm_handler, action_handler, ActionForm, BUSY_STATUS,
    THINKING_STATUS,
};

#[cfg(test)]
mod actions_tests;
