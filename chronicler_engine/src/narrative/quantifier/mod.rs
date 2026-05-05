//! Quantifier module: determines NPC presence and player movement from LLM responses.

pub mod backends;
pub mod core;
pub mod parser;
pub mod prompt;
pub mod types;

pub use backends::{
    MockQuantifierBackend, QuantifierBackendTrait, RealQuantifierBackend, get_quantifier_backend,
    get_quantifier_backend_for,
};
pub use core::determine_npcs_in_room;
pub use parser::{
    compute_npc_events, extract_movement_from_text, parse_quantifier_response,
    parse_quantifier_response_with_movement,
};
pub use prompt::QuantifierPromptBuilder;
pub use types::{
    MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcEventType, QuantifierConfidence,
    QuantifierParseResult, QuantifierPromptContext, QuantifierResult, RoomInfo,
};

#[cfg(test)]
mod action_boundary_tests;
#[cfg(test)]
mod tests;
