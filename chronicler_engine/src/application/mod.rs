//! [DOC: docs/architecture/system.md]

pub mod action_pipeline;
pub mod application_service;
pub mod context;
pub mod game_lifecycle;
pub mod game_service;
pub mod message_editing;
pub mod query_handlers;

pub use application_service::{
    ApplicationError, DebugStateView, DefaultApplicationService, ProcessActionResult,
};

pub use game_lifecycle::GameLifecycleService;
pub use message_editing::MessageEditingService;
pub use query_handlers::QueryHandlers;

#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod query_handlers_tests;
