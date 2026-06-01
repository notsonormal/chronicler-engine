//! [DOC: docs/architecture/system.md]

mod fragments;
mod handlers;
mod template;

pub use handlers::{
    activate_preset_handler, delete_preset_handler, duplicate_preset_handler,
    edit_preset_form_handler, panel_handler, preset_card_handler, save_preset_handler,
    update_preset_handler, view_preset_form_handler,
};
pub use template::PromptPresetsTemplate;

#[cfg(test)]
mod handlers_tests;
#[cfg(test)]
mod template_tests;
