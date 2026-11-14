//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Story memory writer — rewrites the story memory when Messages leave the history.
pub mod agent;
pub mod prompt;

pub use agent::StoryMemoryWriter;

#[cfg(test)]
mod agent_tests;
