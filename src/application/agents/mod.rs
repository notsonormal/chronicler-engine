//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent registry and trait definitions

pub mod quantifier;
pub mod registry;
pub mod trait_def;

pub use crate::domain::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, ExecutionPhase, StatePatch,
};
pub use trait_def::Agent;

#[cfg(test)]
mod registry_tests;
