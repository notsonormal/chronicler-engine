//! [DOC: docs/architecture/system.md]

mod action_pipeline;
mod actions;
mod context;
mod helpers;
mod retry;
mod service;

pub use context::GameServiceContext;
pub use helpers::{delete_and_remove_message, map_llm_error, persist_new_messages};
pub use service::{DefaultGameService, GameService};

#[cfg(test)]
mod helpers_tests;
#[cfg(test)]
mod retry_tests;
