//! [DOC: docs/system/dashboard.md]
//! Fragment renderers (re-exports from submodules)

pub mod fragment_renderers;
pub mod response;

pub use fragment_renderers::*;
pub use response::*;

#[cfg(test)]
mod fragment_renderers_tests;

#[cfg(test)]
mod response_tests;
