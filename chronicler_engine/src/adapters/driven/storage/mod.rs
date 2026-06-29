//! [DOC: docs/system/storage.md]
//! Storage layer and database access

pub mod backend;
pub mod db;
pub mod mappers;
pub mod models;

pub use backend::*;

#[cfg(test)]
mod db_tests;
