use crate::model::character::NpcCard;
use crate::model::map::Room;
use crate::model::state::MessageEntry;

pub use crate::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};

pub struct RoomInfo {
    pub id: String,
    pub name: String,
}

pub struct QuantifierPromptContext<'a> {
    pub room: &'a Room,
    pub previous_room_npcs: &'a [NpcCard],
    pub all_known_npcs: &'a [NpcCard],
    pub all_rooms: &'a [RoomInfo],
    pub player_name: &'a str,
    pub recent_history: &'a [MessageEntry],
    pub player_action: &'a str,
    pub quantifier_prompt_override: Option<String>,
}
