//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Settings askama templates.

pub mod settings;

pub use self::settings::SettingsTemplate;

#[cfg(test)]
mod settings_tests;
