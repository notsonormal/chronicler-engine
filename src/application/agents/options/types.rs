//! [DOC: docs/diataxis/reference/narrative/options.md]
//! Options agent type definitions

use crate::domain::model::character::NpcCard;
use crate::domain::model::map::Room;
use crate::domain::model::state::message_types::StoryRow;

#[derive(Clone)]
pub struct OptionsPromptContext<'a> {
    pub room: &'a Room,
    pub npcs_in_area: &'a [NpcCard],
    pub recent_history: &'a [StoryRow],
    pub player_name: &'a str,
    pub options_prompt_override: Option<String>,
    pub option_count: u32,
}
