//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent type definitions

use crate::domain::model::character::{CharacterCard, PersonaCard};
use crate::domain::model::map::Room;
use crate::domain::model::state::message_types::StoryRow;

#[derive(Clone)]
pub struct OptionsPromptContext<'a> {
    pub room: &'a Room,
    pub characters_present: &'a [CharacterCard],
    pub recent_history: &'a [StoryRow],
    pub persona: &'a PersonaCard,
    pub options_prompt_override: Option<String>,
    pub option_count: u32,
}
