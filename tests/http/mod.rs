//! HTTP test binary root: real-request integration tests for action handlers, fragment rendering, connections UI, debug endpoints, server wiring, and the per-endpoint text-check suite.

#[path = "../test_utils/mod.rs"]
mod test_utils;
pub use test_utils::settings_guard::SettingsTestGuard;
pub use test_utils::TEST_PERSONA;

mod actions;
mod games_create;
mod games_delete;
mod games_switch;
mod prompt_presets;
mod reset;
mod retrigger;
mod settings;
mod story_log;
mod swipe_new;
mod test_helpers;

mod requires_migration;
