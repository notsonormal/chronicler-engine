//! [DOC: docs/architecture/system.md]

mod actions;
mod context;
mod helpers;
mod retry;
mod service;

pub use context::GameServiceContext;
pub use service::{DefaultGameService, GameService};

#[cfg(test)]
mod helpers_tests;
#[cfg(test)]
mod retry_tests;
