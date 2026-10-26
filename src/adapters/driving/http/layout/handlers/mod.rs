//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Layout route handlers (partials + status endpoints).

mod endpoints;

pub use endpoints::{
    busy_answer, busy_status_handler, failure_answer, idle_answer, changes_handler,
    character_headshots_fragment, failure_banner_fragment, header_fragment,
    llm_message_record_fragment, llm_messages_fragment, options_dock_fragment,
    render_status_display, reset_busy_handler, story_log_fragment, visual_sidebar_fragment,
};

#[cfg(test)]
mod endpoints_tests;
