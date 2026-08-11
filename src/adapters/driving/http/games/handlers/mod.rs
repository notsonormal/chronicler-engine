//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games route handlers.

mod games;

pub use self::games::{
    create_game_handler, delete_game_handler, list_games_fragment, switch_game_handler,
    CreateGameForm,
};

#[cfg(test)]
mod games_tests;
