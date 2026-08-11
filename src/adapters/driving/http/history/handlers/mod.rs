//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! History route handlers.

mod history;

pub use history::{delete_history_handler, edit_history_handler, EditHistoryForm};

#[cfg(test)]
mod history_tests;
