//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! HTTP utility modules.

pub mod error;
pub mod fragment;
pub mod handler_helpers;
pub mod locks;
#[cfg(test)]
mod locks_tests;
pub mod port_utils;
pub mod response;
pub mod template_helpers;
pub mod view_mappers;
pub mod view_models;

pub use locks::{read_lock_or_recover, write_lock_or_recover};

#[cfg(test)]
mod response_tests;

#[cfg(test)]
mod view_mappers_tests;

#[cfg(test)]
mod view_models_tests;
