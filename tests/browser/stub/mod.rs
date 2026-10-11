//! Stub-browser tests: browser-only behaviour against a fake engine.

// The fake is `tests/test_utils/stub_server.rs`. Tier placement, the readiness
// gates, and the mirror convention are in `tests/STRATEGY.md`.

#[allow(unused_imports)]
pub use super::*;

mod announcements;
mod dashboard;
mod failure_display;
mod form_failures;
mod games;
mod layout;
mod llm_messages;
mod options;
mod settings;
mod slash_menu;
mod story_log;
mod story_log_edit;
mod support;
mod swipes;
