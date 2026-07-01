//! [DOC: docs/system/game_flow.md]
//! Application layer services and game flow orchestration

pub mod action_pipeline;
pub mod agents;
pub mod application_service;
pub mod context;
pub mod game_service;
pub mod llm_recorder;
pub mod llm_sanitizer;
pub mod message_editing;
pub mod narrative_prompt;
pub mod ports;
pub mod query_handlers;
pub(crate) mod spawn;
pub mod text_check_service;

pub use application_service::{
    ApplicationError, DebugStateView, DefaultApplicationService, ProcessActionResult,
};

pub use context::{
    delete_and_remove_message, map_llm_error, save_message_and_snapshot, GameServiceContext,
};
pub use game_service::GameService;
pub use message_editing::{delete_last, edit_history, retrigger, retry, switch_swipe};
pub use query_handlers::*;
pub(crate) use spawn::spawn_pipeline_task;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod llm_sanitizer_tests;

#[cfg(test)]
mod query_handlers_tests;
