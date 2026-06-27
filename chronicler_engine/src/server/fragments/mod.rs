//! [DOC: docs/system/dashboard.md]
//! UI fragment modules

mod actions;
mod endpoints;
mod generation_guard;
#[cfg(test)]
mod generation_guard_tests;
mod history;
mod misc;
pub mod renderers;

pub use actions::{ActionForm, action_check_handler, action_confirm_handler, action_handler};
pub use endpoints::{
    action_area_fragment, character_headshots_fragment, generating_status_handler, header_fragment,
    hints_handler, llm_messages_fragment, reset_generating_handler, status_ready_handler,
    story_log_fragment, visual_sidebar_fragment,
};
pub use generation_guard::GenerationGuard;
pub use history::{EditHistoryForm, delete_history_handler, edit_history_handler};
pub use misc::{
    check_text_handler, reset_handler, retrigger_handler, retry_handler, switch_swipe_handler,
};
pub use renderers::{
    app_err_to_response, app_err_to_tuple, bad_request, ctx_or_error, html_escape, internal_error,
    ok, ok_refresh, render_error, render_llm_messages, service_unavailable,
    service_unavailable_generating,
};

#[cfg(test)]
mod actions_tests;
#[cfg(test)]
mod endpoints_tests;

#[cfg(test)]
mod history_tests;
