//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! Narration: the generation a Job's narration call makes

pub mod narrator;

pub use narrator::{NarrationValues, Narrated, Narrator};

#[cfg(test)]
mod narrator_tests;
