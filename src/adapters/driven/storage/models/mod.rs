//! [DOC: docs/diataxis/reference/storage.md]
//! Database schema entity definitions

pub mod character;
pub mod game;
pub mod game_state_snapshot;
pub mod llm_message;
pub mod map;
pub mod message;
pub mod persona;
pub mod prompt_preset;
pub mod settings;
pub mod swipe;
pub mod world;

#[cfg(test)]
mod message_tests;
