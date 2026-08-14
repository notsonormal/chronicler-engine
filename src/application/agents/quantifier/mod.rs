//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier agent system
pub mod agent;
pub use utils::parser;
pub mod prompt;
pub mod types;
pub(crate) mod utils;

pub use agent::QuantifierAgent;
pub use utils::orchestration::determine_npcs_in_room;
pub use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcTransitionType,
    QuantifierConfidence, QuantifierParseResult, QuantifierResult,
};
pub use prompt::QuantifierPromptBuilder;
pub use types::{QuantifierPromptContext, RoomInfo};

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod prompt_tests;
