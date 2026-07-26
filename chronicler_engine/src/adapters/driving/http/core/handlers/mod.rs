//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Core route handlers (index, debug, retry, retrigger, game_control, swipe, text_check).

mod debug;
mod game_control;
mod index;
mod retrigger;
mod retry;
mod swipe;
mod text_check;

pub use self::{
    debug::{
        debug_backend_handler, debug_is_generating_handler, debug_state_handler,
        DebugBackendResponse,
    },
    game_control::reset_handler,
    index::index_handler,
    retry::retry_handler,
    retrigger::retrigger_handler,
    swipe::switch_swipe_handler,
    text_check::{check_text_handler, CheckTextForm},
};

#[cfg(test)]
mod debug_tests;
#[cfg(test)]
mod retrigger_tests;
#[cfg(test)]
mod retry_tests;
#[cfg(test)]
mod swipe_tests;
