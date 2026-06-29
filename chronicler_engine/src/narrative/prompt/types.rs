//! [DOC: docs/system/prompt_system.md]
//! Prompt type definitions

use crate::domain::model::character::{NpcCard, PlayerCard};
use crate::domain::model::map::Room;
use crate::domain::model::state::message_types::MessageEntry;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::world::WorldCard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptLayer {
    System,
    GameState,
    NpcCards,
    Player,
    WorldInfo,
    History,
    User,
}

#[derive(Debug, Clone, Copy)]
pub struct NpcContext<'a> {
    pub all_npcs: &'a [NpcCard],
    pub npcs_in_area: &'a [NpcCard],
}

#[derive(Debug, Clone)]
pub struct PromptContext<'a> {
    pub world: &'a WorldCard,
    pub room: &'a Room,
    pub npcs: NpcContext<'a>,
    pub player: &'a PlayerCard,
    pub user_message: &'a str,
    pub history: &'a [MessageEntry],
    pub template_vars: TemplateVars,
}
