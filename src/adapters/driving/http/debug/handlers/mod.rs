//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Debug route handlers.

mod debug;

pub use self::debug::{
    DebugBackendResponse, debug_backend_handler, debug_is_generating_handler, debug_state_handler,
};

#[cfg(test)]
mod debug_tests;
