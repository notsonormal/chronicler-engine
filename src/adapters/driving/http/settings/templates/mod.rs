//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings askama templates.

pub mod settings;

pub use self::settings::{ConnectionFormTemplate, SettingsTemplate, TextCheckCardTemplate};

#[cfg(test)]
mod settings_tests;
