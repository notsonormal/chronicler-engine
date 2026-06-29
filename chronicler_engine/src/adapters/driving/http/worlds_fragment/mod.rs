//! [DOC: docs/system/worlds.md]
//! Worlds management fragment module

mod fragments;
mod handlers;
pub mod template;

pub use fragments::{render_world_edit_form, render_worlds_panel};
pub use handlers::{
    create_world_handler, delete_world_handler, edit_world_form_handler, list_worlds_fragment,
    new_world_form_handler, update_world_handler,
};
