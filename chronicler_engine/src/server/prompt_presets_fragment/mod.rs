//! [DOC: docs/architecture/system.md]

mod fragments;
mod handlers;
mod template;

pub use handlers::{
    activate_preset_handler, delete_preset_handler, edit_preset_form_handler, panel_handler,
    save_preset_handler, update_preset_handler,
};
pub use template::PromptPresetsTemplate;

#[cfg(test)]
mod fragments_tests;
#[cfg(test)]
mod handlers_tests;
