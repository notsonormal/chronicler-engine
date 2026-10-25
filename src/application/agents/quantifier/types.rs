//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier type definitions

use crate::domain::model::character::CharacterCard;
use crate::domain::model::map::Room;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::message_types::StoryRow;

/// How sure the Quantifier is of its answer. It stays inside the Quantifier: the
/// retry and the fallback read it, and the Quantifier result does not carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum QuantifierConfidence {
    /// JSON parsed successfully and all Character IDs are valid.
    High,
    /// Text fallback was used; some valid Character IDs were found.
    Medium,
    /// No valid Character IDs could be extracted; fallback data should be used.
    #[default]
    Low,
}

impl QuantifierConfidence {
    pub(crate) fn is_low(self) -> bool {
        matches!(self, Self::Low)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct QuantifierParseResult {
    pub character_ids: Vec<String>,
    pub confidence: QuantifierConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MovementType {
    /// The player is entering a new room.
    Entering,
    /// The player is leaving the current room.
    Leaving,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct MovementParseResult {
    pub movement_type: Option<MovementType>,
    pub destination: Option<String>,
}

/// One parsed answer of the model: the Characters present and the movement.
#[derive(Debug, Clone, Default)]
pub(crate) struct QuantifierReading {
    pub characters: QuantifierParseResult,
    pub movement: MovementParseResult,
}

impl QuantifierReading {
    pub(crate) fn into_result(self) -> QuantifierResult {
        QuantifierResult {
            present: self.characters.character_ids,
            destination: self.movement.destination,
        }
    }
}

pub struct RoomInfo {
    pub id: String,
    pub name: String,
}

pub struct QuantifierPromptContext<'a> {
    pub room: &'a Room,
    pub previous_characters_present: &'a [CharacterCard],
    pub all_known_characters: &'a [CharacterCard],
    pub all_rooms: &'a [RoomInfo],
    pub player_name: &'a str,
    pub recent_history: &'a [StoryRow],
    pub player_action: &'a str,
    pub quantifier_prompt_override: Option<String>,
}
