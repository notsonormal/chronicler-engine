//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! HTTP server and API endpoints

pub mod app_state;
pub mod debug;
pub mod error;
pub mod fragments;
pub mod games_fragment;
pub mod handlers;
pub mod locks;
pub mod port_utils;
pub mod prompt_presets_fragment;
pub mod router;
pub mod server_impl;
pub mod settings_fragment;
pub mod templates;
pub mod view_models;
pub mod worlds_fragment;

pub use app_state::{AppState, ServerResources};
pub use server_impl::{run_server_with_config, ServerConfig};

#[cfg(test)]
mod locks_tests;

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
