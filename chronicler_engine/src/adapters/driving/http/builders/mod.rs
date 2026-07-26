//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! HTTP builders — composition fns that assemble HTML, headers, and routes.

pub mod connections;
pub mod forms;
pub mod headers;
pub mod presets;
pub mod router;

#[cfg(test)]
mod connections_tests;
#[cfg(test)]
mod presets_tests;
