pub mod agent;
pub mod core;
pub mod parser;
pub mod prompt;
pub mod types;

pub use agent::QuantifierAgent;
pub use core::determine_npcs_in_room;
// compute_npc_events now lives in model::quantifier; re-export for convenience.
pub use crate::model::quantifier::compute_npc_events;
pub use parser::{
    extract_movement_from_text, parse_quantifier_response, parse_quantifier_response_with_movement,
};
pub use prompt::QuantifierPromptBuilder;
// Mechanical types now live in model::quantifier; re-export for convenience.
pub use crate::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcEventType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
pub use types::{QuantifierPromptContext, RoomInfo};

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod prompt_tests;
