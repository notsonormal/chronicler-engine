//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Application layer services and game flow orchestration

pub mod action_pipeline;
pub mod agents;
pub mod application_service;
pub mod arrival_service;
pub mod debug;
pub mod errors;
pub mod game_catalogue;
pub mod game_service;
pub mod game_view_query;
pub mod generation_gate;
pub mod generation_guard;
pub mod llm_message;
pub mod llm_recorder;
pub mod narrative_prompt;
pub mod persistence_gate;
pub mod ports;
pub mod text_check_service;
pub mod utils;
pub mod world_catalogue;

pub use application_service::{
    ApplicationError, DebugStateView, DefaultApplicationService, ProcessActionResult,
};
pub use game_catalogue::GameCatalogue;
pub use game_view_query::GameViewQuery;
pub use world_catalogue::WorldCatalogue;
pub use generation_gate::GenerationGate;
pub use game_service::GameService;
pub use generation_guard::GenerationGuard;
pub use utils::retry::{retrigger, retry};
pub(crate) use utils::spawn::spawn_pipeline_task;

#[cfg(test)]
mod llm_recorder_tests;

#[cfg(test)]
mod application_service_tests;

#[cfg(test)]
mod generation_guard_tests;

#[cfg(test)]
mod text_check_service_tests;
