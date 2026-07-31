//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt type definitions

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::Room;
use crate::domain::model::state::message_types::MessageEntry;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::world::WorldCard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptLayer {
    /// Top-level system instructions and persona rules.
    System,
    /// Current game-state snapshot (room, flags, inventory) injected into the prompt.
    GameState,
    /// NPC card block for NPCs present in the current scene.
    NpcCards,
    /// Persona card block driving narrator voice.
    Persona,
    /// World lore and metadata block.
    WorldInfo,
    /// Prior message history rolling window.
    History,
    /// Current player input turn.
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
    pub persona: &'a PersonaCard,
    pub user_message: &'a str,
    pub history: &'a [MessageEntry],
    pub template_vars: TemplateVars,
}
