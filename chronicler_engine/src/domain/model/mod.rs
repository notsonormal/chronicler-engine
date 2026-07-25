//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Core data models and domain types

pub mod action;
pub mod agent;
pub mod character;
pub mod game;
pub mod llm_backend;
pub mod map;
pub mod message;
pub mod message_history;
pub mod prompt_preset;
pub mod quantifier;
pub mod scenario;
pub mod settings;
pub mod state;
pub mod template;
pub mod trigger;
pub mod world;

#[cfg(test)]
mod action_tests;
#[cfg(test)]
mod character_tests;
#[cfg(test)]
mod game_tests;
#[cfg(test)]
mod llm_backend_tests;
#[cfg(test)]
mod map_tests;
#[cfg(test)]
mod message_history_tests;
#[cfg(test)]
mod message_tests;
#[cfg(test)]
mod prompt_preset_tests;
#[cfg(test)]
mod scenario_tests;
#[cfg(test)]
mod template_tests;
#[cfg(test)]
mod trigger_tests;
#[cfg(test)]
mod world_tests;
