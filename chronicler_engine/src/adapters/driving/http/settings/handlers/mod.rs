//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Settings route handlers.

mod settings;

pub use self::settings::{
    add_connection_handler, connection_card_fragment, delete_connection_handler,
    edit_connection_form, edit_connection_handler, save_settings_handler, save_text_check_handler,
    set_narrator_handler, set_quantifier_handler, settings_panel, ConnectionForm, SettingsForm,
    TextCheckForm,
};

#[cfg(test)]
mod settings_tests;
