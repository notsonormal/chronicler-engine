//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Application layer services and game flow orchestration

pub mod agents;
pub mod arrival_service;
pub mod debug;
pub mod errors;
pub mod games;
pub mod generation;
pub mod llm_message;
pub mod llm_recorder;
pub mod persistence_gate;
pub mod pipeline;
pub mod ports;
pub mod prompting;
pub mod text_check_service;

pub use debug::DebugStateView;
pub use errors::{ApplicationError, ProcessActionResult};
pub use games::{GameCatalogue, GameViewQuery};
pub use generation::{GenerationGate, GenerationGuard};

#[cfg(test)]
mod llm_recorder_tests;

#[cfg(test)]
mod orchestrator_tests;

#[cfg(test)]
mod text_check_service_tests;
