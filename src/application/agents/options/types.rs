//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent type definitions

use crate::domain::model::character::NpcCard;
use crate::domain::model::map::Room;
use crate::domain::model::state::message_types::MessageEntry;

#[derive(Clone)]
pub struct OptionsPromptContext<'a> {
    pub room: &'a Room,
    pub npcs_in_area: &'a [NpcCard],
    pub recent_history: &'a [MessageEntry],
    pub player_name: &'a str,
    /// The active options preset's assembled text (system prompt base).
    pub options_prompt_override: Option<String>,
    pub option_count: u32,
}
