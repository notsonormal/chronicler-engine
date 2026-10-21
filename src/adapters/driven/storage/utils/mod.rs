//! [DOC: docs/diataxis/reference/storage.md]
//! Storage-layer plumbing utilities (datetime parsing, schema migrations).

pub(crate) mod plumbing;
pub(crate) mod records;

#[cfg(test)]
mod plumbing_tests;

pub(crate) use plumbing::{parse_datetime, run_migrations};
pub(crate) use records::require_message_of_game;
