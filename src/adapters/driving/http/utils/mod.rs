//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP utility modules.

pub mod error;
pub mod fragment;
pub mod handler_helpers;
pub mod port_utils;
pub mod response;
pub mod template_helpers;
pub mod view_mappers;
pub mod view_models;

#[cfg(test)]
mod handler_helpers_tests;

#[cfg(test)]
mod response_tests;

#[cfg(test)]
mod template_helpers_tests;

#[cfg(test)]
mod view_mappers_tests;

#[cfg(test)]
mod view_models_tests;
