//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings route handlers.

mod settings;

pub use self::settings::{
    add_connection_handler, delete_connection_handler, edit_connection_form,
    edit_connection_handler, new_connection_form, save_text_check_handler, set_narrator_handler,
    set_quantifier_handler, settings_panel, test_form_connection_handler,
    test_saved_connection_handler, ConnectionForm, RoleForm, TextCheckForm,
};

#[cfg(test)]
mod settings_tests;
