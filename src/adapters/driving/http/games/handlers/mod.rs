//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games route handlers.

mod games;

pub use self::games::{
    create_game_handler, delete_game_handler, list_games_fragment, switch_game_handler,
    switch_game_mode_handler, update_game_posture_handler, update_game_presets_handler,
    CreateGameForm, GameModeForm, GamePostureForm, GamePresetsForm,
};

#[cfg(test)]
mod games_tests;
