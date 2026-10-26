//! Shared test utilities re-exported across all test binaries: `browser`, `convergence`, `screen`, `server`, `settings_guard`, `stub_action_layers`, `wait`, plus the `TEST_WORLD` / `TEST_PERSONA` constants.

#![allow(dead_code)]

pub mod browser;
pub mod convergence;
pub mod html;
pub mod htmx_settle;
pub mod screen;
pub mod screen_part_table;
pub mod server;
pub mod settings_guard;
pub mod stub_action_layers;
pub mod stub_server;
pub mod wait;

#[allow(unused_imports)]
pub use browser::*;
#[allow(unused_imports)]
pub use convergence::*;
#[allow(unused_imports)]
pub use html::*;
#[allow(unused_imports)]
pub use screen::*;
#[allow(unused_imports)]
pub use server::*;
#[allow(unused_imports)]
pub use htmx_settle::*;
#[allow(unused_imports)]
pub use stub_server::*;
#[allow(unused_imports)]
pub use wait::*;
#[allow(unused_imports)]
pub use chronicler_engine::test_support::{make_test_recorder, make_test_recorder_with_storage};

pub const TEST_WORLD: &str = "test";
pub const TEST_PERSONA: &str = "test_player";
pub const CONFIG_PATH: &str = "tests/test_config.json";

/// The selects of the Game's config controls in the Games panel.
pub const GAME_CONFIG_SELECT_IDS: [&str; 7] = [
    "game-narrator-mode",
    "game-narrative-perspective",
    "game-narrative-tense",
    "game-system-preset",
    "game-quantifier-preset",
    "game-impersonate-preset",
    "game-options-preset",
];
