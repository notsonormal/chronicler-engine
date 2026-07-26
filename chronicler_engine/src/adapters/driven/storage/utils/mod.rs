//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]
//! Storage-layer plumbing utilities (datetime parsing, schema migrations).

pub(crate) mod plumbing;

pub(crate) use plumbing::{parse_datetime, run_migrations};
