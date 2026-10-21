//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent hooks: the hook traits and the AgentRunner

pub mod options;
pub mod quantifier;
pub mod runner;
pub mod trait_def;

pub use runner::AgentRunner;
pub use trait_def::{AfterNarrationAgent, EndOfJobAgent};

#[cfg(test)]
mod runner_tests;
