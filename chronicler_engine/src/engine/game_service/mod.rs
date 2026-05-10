//! [DOC: docs/architecture/system.md]

mod actions;
mod context;
mod helpers;
mod retry;
mod service;

pub use context::GameServiceContext;
pub use service::{DefaultGameService, GameService};
