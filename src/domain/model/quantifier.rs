//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Quantifier result and the Character transitions the Game rules derive from it

/// What the Quantifier found in a generation's new text. The Game rules apply it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuantifierResult {
    pub present: Vec<String>,
    /// The room the player walks to, by id or name.
    pub destination: Option<String>,
}

/// Transition type for NPC area changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcTransitionType {
    /// NPC entered the area (was not present, now present).
    Entered,
    /// NPC left the area (was present, now not present).
    Left,
}

/// A single NPC area transition event.
///
/// These events are **engine state** — computed from a diff between previous and current
/// NPC presence, NOT from the quantifier LLM output directly. The quantifier detects
/// which NPCs are in the room; the engine computes Enter/Leave transitions via set diff.
#[derive(Debug, Clone)]
pub struct NpcEvent {
    /// NPC ID that moved.
    pub npc_id: String,
    /// Type of transition.
    pub event_type: NpcTransitionType,
}

/// The Character transitions between two sets of Character ids.
#[derive(Debug, Clone, Default)]
pub struct NpcEventList {
    pub events: Vec<NpcEvent>,
}

impl NpcEventList {
    pub fn from_diff(previous_npc_ids: &[String], current_npc_ids: &[String]) -> Self {
        let previous_set: std::collections::HashSet<_> = previous_npc_ids.iter().collect();
        let current_set: std::collections::HashSet<_> = current_npc_ids.iter().collect();

        let mut events = Vec::new();

        for npc_id in current_npc_ids {
            if !previous_set.contains(npc_id) {
                events.push(NpcEvent {
                    npc_id: npc_id.clone(),
                    event_type: NpcTransitionType::Entered,
                });
            }
        }
        for npc_id in previous_npc_ids {
            if !current_set.contains(npc_id) {
                events.push(NpcEvent {
                    npc_id: npc_id.clone(),
                    event_type: NpcTransitionType::Left,
                });
            }
        }

        Self { events }
    }
}
