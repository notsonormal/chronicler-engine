//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt presets route handlers.

mod prompt_presets;

pub use self::prompt_presets::{
    ActivateQuery, activate_preset_handler, delete_preset_handler, duplicate_preset_handler,
    edit_preset_form_handler, new_preset_form_handler, panel_handler, save_preset_handler,
    update_preset_handler, view_preset_form_handler, NewPresetQuery, PresetForm,
};

#[cfg(test)]
mod prompt_presets_tests;
