//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! HTTP server and API endpoints

pub mod app_state;
pub mod builders;
pub mod debug;
pub mod error;
pub mod fragments;
pub mod games_fragment;
pub mod handlers;
pub mod port_utils;
pub mod prompt_presets_fragment;
pub mod server_impl;
pub mod settings_fragment;
pub mod templates;
pub mod utils;
pub mod view_models;
pub mod worlds_fragment;

pub use app_state::AppState;
pub use server_impl::{run_server_with_config, ServerConfig};
pub use utils::{read_lock_or_recover, write_lock_or_recover};

#[cfg(test)]
mod debug_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod port_utils_tests;
#[cfg(test)]
mod templates_tests;
#[cfg(test)]
mod view_models_tests;
