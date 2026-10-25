//! [DOC: docs/diataxis/reference/storage.md]
//! Storage layer and database access

pub mod changes;
pub mod characters;
pub mod core;
pub mod db;
pub mod docks;
pub mod games;
pub mod llm_messages;
pub mod mappers;
pub mod models;
pub mod personas;
pub mod presets;
pub mod settings;
pub mod stories;
pub mod transaction;
pub mod utils;
pub mod worlds;

pub use core::*;
pub use transaction::{Tx, TxToken, TxWrite};

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
mod personas_tests;
#[cfg(test)]
mod presets_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod worlds_tests;
