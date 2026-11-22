//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Story memory writer — writes the memory entries of the Messages that leave the history.
pub mod agent;
pub mod parser;
pub mod prompt;

pub use agent::StoryMemoryWriter;

#[cfg(test)]
mod agent_tests;
