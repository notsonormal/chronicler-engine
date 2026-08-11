//! [DOC: docs/diataxis/reference/storage.md]
//! Storage layer and database access

pub mod backend;
pub mod db;
pub mod mappers;
pub mod models;
pub mod preset_store;
pub mod utils;

pub use backend::*;
pub use preset_store::PresetStore;

#[cfg(test)]
mod db_tests;
