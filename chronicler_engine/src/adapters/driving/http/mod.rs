//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! HTTP server and API endpoints

pub mod action;
pub mod app_state;
pub mod bootstrap;
pub mod builders;
pub mod core;
pub mod error;
pub mod games;
pub mod history;
pub mod layout;
pub mod prompt_presets;
pub mod settings;
pub mod templates;
pub mod utils;
pub mod view_models;
pub mod worlds;

pub use app_state::AppState;
pub use bootstrap::{run_server_with_config, ServerConfig};
pub use utils::{read_lock_or_recover, write_lock_or_recover};

#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod templates_tests;
#[cfg(test)]
mod view_models_tests;
