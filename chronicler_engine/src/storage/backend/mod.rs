//! [DOC: docs/system/storage.md]
//! Storage backend modules

pub mod characters;
pub mod core;
pub mod games;
pub mod helpers;
pub mod llm_messages;
pub mod messages;
pub mod personas;
pub mod presets;
pub mod settings;
pub mod snapshots;
pub mod swipes;
pub mod worlds;

pub use helpers::empty_to_none;
#[cfg(feature = "testing")]
pub use test_support::{ErrorKind, TestFailureHandle, TestOverride};

#[cfg(feature = "testing")]
pub mod test_support;

#[cfg(test)]
mod characters_tests;
#[cfg(test)]
mod core_tests;
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

pub use core::*;
