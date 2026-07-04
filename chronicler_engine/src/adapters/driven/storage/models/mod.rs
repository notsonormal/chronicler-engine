//! [DOC: docs/system/storage.md]
//! Database schema entity definitions

pub mod character;
pub mod game;
pub mod game_state_snapshot;
pub mod llm_message;
pub mod message;
pub mod persona;
pub mod prompt_preset;
pub mod settings;
pub mod world;

pub use game::DbGame;
pub use game_state_snapshot::DbGameStateSnapshot;
pub use llm_message::DbLlmMessage;
pub use message::DbMessage;
pub use prompt_preset::DbPromptPreset;
pub use world::{DbWorld, DbMap};
pub use persona::DbPersona;
pub use character::DbCharacter;
pub use settings::DbSettings;

#[cfg(test)]
mod message_tests;
