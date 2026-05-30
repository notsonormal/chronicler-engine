use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::Room;
use crate::model::state::MessageEntry;
use crate::model::world::WorldCard;

/// [DOC: docs/system/prompt_system.md]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptLayer {
    /// Layer 0: System prompt - global game rules and AI role
    System,
    /// Layer 1: Game state - current world, location, active quests
    GameState,
    /// Layer 2: NPC cards - active character information
    NpcCards,
    /// Layer 3: Player - character stats, inventory, relationships
    Player,
    /// Layer 4: World info - lore, geography, factions
    WorldInfo,
    /// Layer 5: History - recent conversation/actions (prone to truncation)
    History,
    /// Layer 6: User input - current command/speech
    User,
    /// Layer 7: Phi layer - auxiliary context, reminders, formatting hints
    Phi,
}
#[derive(Debug, Clone)]
pub struct PromptContext<'a> {
    pub world: &'a WorldCard,
    pub room: &'a Room,
    pub all_npcs: &'a [NpcCard],
    pub npcs_in_area: &'a [NpcCard],
    pub player: &'a PlayerCard,
    pub user_message: &'a str,
    pub history: &'a [MessageEntry],
}
