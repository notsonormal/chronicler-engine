//! [DOC: docs/system/storage.md]
//! Storage layer and database access

pub mod backend;
pub mod db;
pub mod mappers;
pub mod models;
pub mod snapshot_blob;

pub use backend::*;

#[cfg(test)]
mod db_tests;
#[cfg(test)]
mod snapshot_blob_tests;
