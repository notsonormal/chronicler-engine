//! [DOC: docs/system/game_flow.md]
//! Quantifier types for narrative evaluation

use crate::model::agent::Confidence;

/// Confidence level of a quantifier or event detection result.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum QuantifierConfidence {
    /// JSON parsed successfully and all NPC IDs are valid.
    High,
    /// Text fallback was used; some valid NPC IDs were found.
    Medium,
    /// No valid NPC IDs could be extracted; fallback data should be used.
    #[default]
    Low,
}

impl QuantifierConfidence {
    pub fn is_high(&self) -> bool {
        matches!(self, Self::High)
    }
}

impl From<Confidence> for QuantifierConfidence {
    fn from(c: Confidence) -> Self {
        match c {
            Confidence::High => Self::High,
            Confidence::Medium => Self::Medium,
            Confidence::Low => Self::Low,
        }
    }
}

impl From<QuantifierConfidence> for Confidence {
    fn from(c: QuantifierConfidence) -> Self {
        match c {
            QuantifierConfidence::High => Self::High,
            QuantifierConfidence::Medium => Self::Medium,
            QuantifierConfidence::Low => Self::Low,
        }
    }
}

/// NPC IDs detected as present in the current room.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuantifierParseResult {
    pub npc_ids: Vec<String>,
    /// How confident the quantifier is in this result.
    pub confidence: QuantifierConfidence,
}

impl QuantifierParseResult {
    pub fn is_high(&self) -> bool {
        matches!(self.confidence, QuantifierConfidence::High)
    }
}

/// Type of movement detected by the quantifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementType {
    /// Player is entering a new room.
    Entering,
    /// Player is already in a room.
    In,
    /// Player is leaving the current room.
    Leaving,
}

/// Movement intent detected by the quantifier, if any.
#[derive(Debug, Clone, Default)]
pub struct MovementParseResult {
    /// Type of movement detected, if any.
    pub movement_type: Option<MovementType>,
    /// Destination room ID or name, if detected.
    pub destination: Option<String>,
    /// Confidence level of the movement detection.
    pub confidence: QuantifierConfidence,
}

/// Combined result of scene quantification: NPC presence + movement.
#[derive(Debug, Clone, Default)]
pub struct QuantifierResult {
    /// NPCs detected as present in the room.
    pub npcs: QuantifierParseResult,
    /// Movement intent detected, if any.
    pub movement: MovementParseResult,
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

/// Collection of NPC transition events with aggregate confidence.
#[derive(Debug, Clone, Default)]
pub struct NpcEventList {
    /// Detected movement events.
    pub events: Vec<NpcEvent>,
    /// Confidence in the event detection.
    pub confidence: QuantifierConfidence,
}
/// Computes NPC transition events by diffing previous and current NPC presence.
pub fn compute_npc_events(previous_npc_ids: &[String], current_npc_ids: &[String]) -> NpcEventList {
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

    let confidence = if !events.is_empty() {
        QuantifierConfidence::Medium
    } else {
        QuantifierConfidence::Low
    };

    NpcEventList { events, confidence }
}
