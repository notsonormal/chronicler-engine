//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt type definitions

use crate::domain::model::character::CharacterCard;

#[derive(Debug, Clone, Copy)]
pub struct CharacterContext<'a> {
    pub all_characters: &'a [CharacterCard],
    pub characters_present: &'a [CharacterCard],
}
