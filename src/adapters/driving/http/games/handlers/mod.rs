//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Games route handlers.

mod games;

pub use self::games::{
    render_games_panel, create_game_handler, delete_game_handler, new_game_form_handler,
    rename_game_form_handler, rename_game_handler, switch_game_handler, switch_game_mode_handler,
    update_game_posture_handler, update_game_presets_handler, CreateGameForm, GameModeForm,
    GamePostureForm, GamePresetsForm, RenameGameForm,
};

#[cfg(test)]
mod games_tests;
