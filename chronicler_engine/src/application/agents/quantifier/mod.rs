//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier agent system

//! Quantifier agent module — three responsibilities:
//! 1. **Detection**: Analyze LLM narration to detect NPC presence and movement intent
//! 2. **Scene quantification**: Produce QuantifierResult (NPC IDs + movement destination)
//! 3. **Event generation**: Compute NPC Enter/Leave events via state diff (compute_npc_events)
//!
pub mod agent;
pub mod orchestration;
pub mod parser;
pub mod prompt;
pub mod types;

pub use agent::QuantifierAgent;
pub use orchestration::determine_npcs_in_room;
// compute_npc_events now lives in model::quantifier; re-export for convenience.
pub use crate::domain::model::quantifier::compute_npc_events;
pub use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcTransitionType,
    QuantifierConfidence, QuantifierParseResult, QuantifierResult,
};
pub use parser::{parse_quantifier_response, parse_quantifier_response_with_movement};
pub use prompt::QuantifierPromptBuilder;
pub use types::{QuantifierPromptContext, RoomInfo};

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod orchestration_tests;
#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod prompt_tests;
