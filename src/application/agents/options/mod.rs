//! [DOC: docs/diataxis/reference/narrative/options.md]
//! Options agent system — generates the pickable next-action option set.
pub mod agent;
pub mod prompt;
pub(crate) mod types;
pub(crate) mod utils;

pub use agent::OptionsAgent;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod prompt_tests;
