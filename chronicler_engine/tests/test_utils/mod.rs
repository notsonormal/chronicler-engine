//! [DOC: docs/reference/testing.md]

#![allow(dead_code)]

pub mod browser;
pub mod server;
pub mod wait;

pub use browser::*;
pub use server::*;
pub use wait::*;

pub const TEST_WORLD: &str = "test";
pub const CONFIG_PATH: &str = "tests/test_config.json";
