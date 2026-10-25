//! [DOC: docs/diataxis/reference/storage.md]
//! The dock record: the offered set and the one Options failure of a Game

use serde::{Deserialize, Serialize};

/// Held apart from the Game record, because the config routes write that record
/// with no busy check.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DockRecord {
    #[serde(default)]
    pub offered: Vec<String>,
    /// Present when a reason line stands in for the offered set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<OptionsFailure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionsFailure {
    pub reason: String,
    /// The raw text behind the details disclosure.
    pub raw: String,
}
