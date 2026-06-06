//! [DOC: docs/system/game_flow.md]
//! Application layer services and game flow orchestration

pub mod action_pipeline;
pub mod application_service;
pub mod context;
pub mod game_lifecycle;
pub mod game_service;
#[cfg(test)]
mod game_service_tests;
pub mod message_editing;
pub mod query_handlers;

pub use application_service::{
    ApplicationError, DebugStateView, DefaultApplicationService, ProcessActionResult,
};

pub use context::{
    delete_and_remove_message, map_llm_error, save_message_and_snapshot, GameServiceContext,
};
pub use game_service::GameService;
pub use message_editing::MessageEditingService;
pub use query_handlers::QueryHandlers;

#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod message_editing_tests;
#[cfg(test)]
mod query_handlers_tests;
