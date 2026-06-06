//! [DOC: docs/system/agent_system.md]

pub mod quantifier;
pub mod registry;
pub mod trait_def;

pub use crate::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, ExecutionPhase, StatePatch,
};
pub use trait_def::Agent;

#[cfg(test)]
mod registry_tests;
