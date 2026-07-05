//! [DOC: docs/reference/testing.md]
//! Shared test utilities re-exported across all test binaries: `browser`, `server`, `settings_guard`, `wait`, plus the `TEST_WORLD` / `TEST_PERSONA` constants.

#![allow(dead_code)]

pub mod browser;
pub mod server;
pub mod settings_guard;
pub mod wait;

#[allow(unused_imports)]
pub use browser::*;
#[allow(unused_imports)]
pub use server::*;
#[allow(unused_imports)]
pub use wait::*;
#[allow(unused_imports)]
pub use chronicler_engine::test_support::{make_test_recorder, make_test_recorder_with_storage};

pub const TEST_WORLD: &str = "test";
pub const TEST_PERSONA: &str = "test_player";
pub const CONFIG_PATH: &str = "tests/test_config.json";
