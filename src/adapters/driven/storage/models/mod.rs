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

pub use character::DbCharacter;
pub use game::DbGame;
pub use game_state_snapshot::DbGameStateSnapshot;
pub use llm_message::DbLlmMessage;
pub use map::DbMap;
pub use message::DbMessage;
pub use persona::DbPersona;
pub use prompt_preset::DbPromptPreset;
pub use settings::DbSettings;
pub use swipe::DbSwipe;
pub use world::DbWorld;

#[cfg(test)]
mod message_tests;
