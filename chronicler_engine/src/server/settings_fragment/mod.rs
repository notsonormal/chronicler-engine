//! [DOC: docs/architecture/system.md]

mod fragments;
mod handlers;
mod template;

pub use handlers::{
    add_connection_handler, connection_card_fragment, delete_connection_handler,
    edit_connection_form, edit_connection_handler, save_settings_handler, save_text_check_handler,
    set_narrator_handler, set_quantifier_handler, settings_panel,
};
pub use template::{SettingsTemplate, parse_api_key};

#[cfg(test)]
mod template_tests;
