//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Trigger reference of an event record, and the event record of a fired Trigger

use serde::{Deserialize, Serialize};

/// Names a Trigger by its Character and its place on that Character's Trigger list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerRef {
    /// The stored key is `npc_id`.
    #[serde(rename = "npc_id")]
    pub character_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
}

/// The record of a fired Trigger. The Swipe state of the Swipe that fired it
/// holds it, and the event Narration is written from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub trigger: TriggerRef,
    /// The name the event header shows. The stored key is `name`.
    #[serde(rename = "name")]
    pub header_name: String,
    /// The Trigger's narration direction, with the Persona name already in it.
    pub direction: String,
}
