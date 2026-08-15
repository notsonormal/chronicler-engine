//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt type definitions

use crate::domain::model::character::NpcCard;

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
