//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Chat window route handlers.

pub mod chat_window;

pub use self::chat_window::{
    CheckTextForm, check_text_handler, index_handler, reset_handler, retrigger_handler,
    retry_handler, switch_swipe_handler,
};

#[cfg(test)]
mod chat_window_tests;
