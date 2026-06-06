//! [DOC: docs/system/storage.md]

pub mod characters;
pub mod core;
pub mod games;
pub mod llm_messages;
pub mod messages;
pub mod personas;
pub mod presets;
pub mod settings;
pub mod snapshots;
pub mod swipes;
pub mod worlds;

#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod games_tests;
#[cfg(test)]
mod llm_messages_tests;
#[cfg(test)]
mod messages_tests;
#[cfg(test)]
mod presets_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod snapshots_tests;
#[cfg(test)]
mod swipes_tests;

pub use core::*;
