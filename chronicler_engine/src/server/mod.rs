pub mod app_state;
pub mod debug;
pub mod fragments;
pub mod handlers;
pub mod port_utils;
pub mod prompt_presets_fragment;
pub mod router;
pub mod server_impl;
pub mod settings_fragment;
pub mod templates;
pub mod view_models;

pub use app_state::{AppState, ServerConfig, ServerResources};
pub(crate) use router::build_router;
pub use router::create_app_with_state;
pub use server_impl::run_server_with_config;

#[cfg(test)]
mod debug_tests;
#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod port_utils_tests;
#[cfg(test)]
mod templates_tests;
#[cfg(test)]
mod view_models_tests;
