//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier agent system
pub mod agent;
pub use utils::parser;
pub mod prompt;
pub mod types;
pub(crate) mod utils;

pub use agent::QuantifierAgent;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod prompt_tests;
