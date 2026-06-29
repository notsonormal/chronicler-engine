//! [DOC: docs/system/dashboard.md]
//! Games fragment module

mod handlers;
mod template;

pub use handlers::{create_game_handler, delete_game_handler, list_games_fragment, switch_game_handler};

#[cfg(test)]
mod handlers_tests;
