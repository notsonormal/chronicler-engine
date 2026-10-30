//! [DOC: docs/diataxis/reference/game_flow.md]
//! Worlds route handlers.

mod worlds;

pub use self::worlds::{
    render_worlds_panel, create_world_handler, delete_world_handler, edit_world_form_handler,
    new_world_form_handler, update_world_handler, update_world_posture_handler, WorldForm,
    WorldPostureForm,
};

#[cfg(test)]
mod worlds_tests;
