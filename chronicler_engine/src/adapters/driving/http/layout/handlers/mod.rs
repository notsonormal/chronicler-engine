//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Layout route handlers (partials + status endpoints).

mod endpoints;

pub use endpoints::{
    action_area_fragment, character_headshots_fragment, generating_status_handler, header_fragment,
    llm_messages_fragment, reset_generating_handler, status_ready_handler, story_log_fragment,
    visual_sidebar_fragment,
};

#[cfg(test)]
mod endpoints_tests;
