//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt type definitions

use crate::domain::model::character::NpcCard;

#[derive(Debug, Clone, Copy)]
pub struct NpcContext<'a> {
    pub all_npcs: &'a [NpcCard],
    pub npcs_in_area: &'a [NpcCard],
}
