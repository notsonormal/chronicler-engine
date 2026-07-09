//! [DOC: docs/system/game_flow.md]
//! Application layer services and game flow orchestration

pub mod action_pipeline;
pub mod agents;
pub mod application_service;
pub mod arrival_service;
pub mod debug;
pub mod errors;
pub mod game_catalogue;
pub mod game_service;
pub mod generation_gate;
pub mod generation_guard;
pub mod llm_recorder;
pub mod llm_sanitizer;
pub mod mappers;
pub mod message_editing;
pub mod narrative_prompt;
pub mod persistence_gate;
pub mod ports;
pub mod query_handlers;
pub(crate) mod scenario;
pub(crate) mod spawn;
pub mod text_check_service;
pub mod world_catalogue;

pub use application_service::{
    ApplicationError, DebugStateView, DefaultApplicationService, ProcessActionResult,
    load_messages_with_swipes, map_llm_error,
};
pub use game_catalogue::GameCatalogue;
pub use world_catalogue::WorldCatalogue;
pub use generation_gate::GenerationGate;
pub use persistence_gate::WorldSnapshot;
pub use game_service::GameService;
pub use generation_guard::GenerationGuard;
pub use message_editing::{delete_last, edit_history, retrigger, retry, switch_swipe};
pub use query_handlers::*;
pub(crate) use spawn::spawn_pipeline_task;

#[cfg(test)]
mod is_generating_invariant_tests;

#[cfg(test)]
mod llm_recorder_tests;

#[cfg(test)]
mod llm_sanitizer_tests;

#[cfg(test)]
mod query_handlers_tests;

#[cfg(test)]
mod text_check_service_tests;
