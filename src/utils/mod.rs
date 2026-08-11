//! [DOC: docs/diataxis/reference/coding_standards/guardrails.md]
//! Top-level utility module — generic helpers that don't belong to a single domain.

pub mod cli;
pub mod settings;

#[cfg(test)]
mod cli_tests;

#[cfg(test)]
mod settings_tests;
