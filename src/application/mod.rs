//! [DOC: docs/diataxis/reference/game_flow.md]
//! Application layer services and game flow orchestration

pub mod agents;
pub mod arrival_service;
#[cfg(test)]
mod arrival_service_tests;
pub mod debug;
pub mod errors;
pub mod games;
pub mod generation;
pub mod llm_recorder;
pub mod message_service;
pub mod persona_catalogue;
pub mod pipeline;
pub mod ports;
pub mod prompt_preset_service;
pub mod prompting;
pub mod settings_service;
pub mod text_check_service;
pub(crate) mod utils;
pub mod world_catalogue;

pub use debug::DebugStateView;

#[cfg(test)]
mod llm_recorder_tests;

#[cfg(test)]
mod message_service_tests;

#[cfg(test)]
mod persona_catalogue_tests;

#[cfg(test)]
mod prompt_preset_service_tests;

#[cfg(test)]
mod settings_service_tests;

#[cfg(test)]
mod text_check_service_tests;

#[cfg(test)]
mod world_catalogue_tests;
