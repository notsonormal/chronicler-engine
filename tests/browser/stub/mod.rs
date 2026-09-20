//! Stub-browser tests: browser-only behaviour against a fake engine.

// The fake is `tests/test_utils/stub_server.rs` — the real dashboard shell
// plus canned fragments, with no engine process behind it. These tests avoid
// the server boot cost the full-stack tier pays.
//
// Tier placement (`tests/STRATEGY.md`): if the server behind the behaviour
// were fake, does the behaviour change? No -> this tier. Yes -> the full-stack
// browser tier (`tests/browser/<surface>.rs`). A test that asserts
// server-derived content belongs in the HTTP tier instead.
//
// Files mirror the specs: `stub/<surface>.rs` covers the stub-tier scenarios
// of `docs/specs/browser_<surface>.md`.

#[allow(unused_imports)]
pub use super::*;

mod dashboard;
mod invariants;
mod options;
mod slash_menu;
mod story_log;
