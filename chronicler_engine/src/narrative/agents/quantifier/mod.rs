pub mod agent;
pub mod core;
pub mod parser;
pub mod prompt;
pub mod types;

pub use agent::QuantifierAgent;
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
pub(crate) mod test_support;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod prompt_tests;
