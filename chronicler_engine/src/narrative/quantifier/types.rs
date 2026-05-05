use crate::model::character::NpcCard;
use crate::model::map::Room;
use crate::model::state::LogEntry;

/// Confidence level of the quantifier's NPC presence detection.
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

/// [DOC: docs/system/navigation.md]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantifierParseResult {
    pub npc_ids: Vec<String>,
    /// How confident the quantifier is in this result.
    pub confidence: QuantifierConfidence,
}

/// Basic room information for the quantifier prompt.
pub struct RoomInfo {
    pub id: String,
    pub name: String,
}

/// Type of movement detected by the quantifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementType {
    /// Player is entering a new room ("I walk through the gate", "enter the kitchen")
    Entering,
    /// Player is already in a room (contextual, rarely used)
    In,
    /// Player is leaving the current room ("I leave the house", "go outside")
    Leaving,
}

/// [DOC: docs/system/navigation.md]
#[derive(Debug, Clone)]
pub struct MovementParseResult {
    /// Type of movement detected, if any.
    pub movement_type: Option<MovementType>,
    /// Destination room ID or name, if detected.
    pub destination: Option<String>,
    /// Confidence level of the movement detection.
    pub confidence: QuantifierConfidence,
}

/// [DOC: docs/system/llm_processing.md]
#[derive(Debug, Clone)]
pub struct QuantifierResult {
    /// NPCs detected as present in the room.
    pub npcs: QuantifierParseResult,
    /// Movement intent detected, if any.
    pub movement: MovementParseResult,
}

/// NPC movement event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcEventType {
    /// NPC entered the area (was not present, now present).
    Entered,
    /// NPC left the area (was present, now not present).
    Left,
}

#[derive(Debug, Clone)]
pub struct NpcEvent {
    /// NPC ID that moved.
    pub npc_id: String,
    /// Type of movement event.
    pub event_type: NpcEventType,
}

/// List of NPC movement events with confidence level.
#[derive(Debug, Clone, Default)]
pub struct NpcEventList {
    /// Detected movement events.
    pub events: Vec<NpcEvent>,
    /// Confidence in the event detection.
    pub confidence: QuantifierConfidence,
}

/// Context needed to build a quantifier prompt.
pub struct QuantifierPromptContext<'a> {
    pub room: &'a Room,
    pub previous_room_npcs: &'a [NpcCard],
    pub all_known_npcs: &'a [NpcCard],
    pub all_rooms: &'a [RoomInfo],
    pub player_name: &'a str,
    pub recent_history: &'a [LogEntry],
    pub player_action: &'a str,
}
