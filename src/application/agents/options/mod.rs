//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent system — generates the Options to pick from.
pub mod agent;
pub mod prompt;
pub(crate) mod types;
pub(crate) mod utils;

pub use agent::OptionsGeneratorAgent;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod prompt_tests;
