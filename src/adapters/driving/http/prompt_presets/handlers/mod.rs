//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt presets route handlers.

mod prompt_presets;

pub use self::prompt_presets::{
    activate_preset_handler, delete_preset_handler, duplicate_preset_handler,
    edit_preset_form_handler, panel_handler, preset_card_handler, save_preset_handler,
    update_preset_handler, view_preset_form_handler, PresetForm,
};

#[cfg(test)]
mod prompt_presets_tests;
