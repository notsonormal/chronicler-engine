//! [DOC: docs/diataxis/reference/storage.md]
//! Storage layer and database access

pub mod characters;
pub mod core;
pub mod db;
pub mod games;
pub mod in_memory_data;
pub mod llm_messages;
pub mod mappers;
pub mod messages;
pub mod models;
pub mod personas;
pub mod presets;
pub mod settings;
pub mod snapshots;
pub mod swipes;
pub mod utils;
pub mod worlds;

pub use core::*;
pub use in_memory_data::*;

#[cfg(feature = "testing")]
pub use test_support::{TestFailureHandle, TestOverride};

#[cfg(feature = "testing")]
mod test_support;

#[cfg(test)]
mod characters_tests;
#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod db_tests;
#[cfg(test)]
mod games_tests;
#[cfg(test)]
mod llm_messages_tests;
#[cfg(test)]
mod messages_tests;
#[cfg(test)]
mod personas_tests;
#[cfg(test)]
mod presets_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod snapshots_tests;
#[cfg(test)]
mod swipes_tests;
#[cfg(test)]
mod worlds_tests;
