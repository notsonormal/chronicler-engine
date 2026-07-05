//! Browser test binary root (Playwright-driven): editing, form interaction, DOM structure, and trigger-driven narration against a real running server.

#[path = "../test_utils/mod.rs"]
mod test_utils;
pub use test_utils::*;

mod editing;
mod interaction;
mod structure;
mod trigger;
