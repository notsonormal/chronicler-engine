//! Stub-browser tests: browser-only behaviour against a fake engine.

// The fake is `tests/test_utils/stub_server.rs`. Tier placement, the readiness
// gates, and the mirror convention are in `tests/STRATEGY.md`.

#[allow(unused_imports)]
pub use super::*;

mod dashboard;
mod invariants;
mod options;
mod slash_menu;
mod story_log;
