//! [DOC: docs/reference/testing.md]

mod test_utils;
use test_utils::*;

#[path = "browser/editing.rs"]
mod editing;
#[path = "browser/interaction.rs"]
mod interaction;
#[path = "browser/structure.rs"]
mod structure;
#[path = "browser/trigger.rs"]
mod trigger;
