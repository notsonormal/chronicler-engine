//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Trigger reference of an event record, and the event record of a fired Trigger

use serde::{Deserialize, Serialize};

/// Names a Trigger by its Character and its place on that Character's Trigger list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerRef {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
}

/// The record of a fired Trigger: the trigger context of the Swipe that fired it,
/// and the record of the event Narration written from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub trigger: TriggerRef,
    /// The name the event header shows. The stored key stays `name`.
    #[serde(rename = "name")]
    pub header_name: String,
    /// The Trigger's narration direction, with the Persona name already in it.
    pub direction: String,
}
