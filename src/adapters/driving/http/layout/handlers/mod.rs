//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Layout route handlers (partials + status endpoints).

mod action_controls;
mod endpoints;

pub use action_controls::{
    autoplay_controls_answer, render_autoplay_controls, render_send_control, send_control_answer,
};

pub use endpoints::{
    busy_answer, changes_handler, failure_answer, idle_answer, llm_message_record_fragment,
    render_status_display, reset_busy_handler,
};

#[cfg(test)]
mod endpoints_tests;
