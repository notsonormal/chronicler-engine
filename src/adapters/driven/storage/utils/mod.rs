//! [DOC: docs/diataxis/reference/storage.md]
//! Storage-layer plumbing utilities (datetime parsing, schema migrations).

pub(crate) mod plumbing;

#[cfg(test)]
mod plumbing_tests;

pub(crate) use plumbing::{parse_datetime, run_migrations};
