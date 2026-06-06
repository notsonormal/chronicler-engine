//! [DOC: docs/system/storage.md]

pub mod backend;
pub mod db;
pub mod mappers;
pub mod models;

pub use backend::*;

#[cfg(test)]
mod db_tests;
