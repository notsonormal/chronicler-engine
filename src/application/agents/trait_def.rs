//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent hook traits: one trait per hook, each giving its own kind of result

use crate::domain::model::agent::{AgentSettings, HookInput, StoryMemoryHookInput, OfferedSet};
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::memory_entry::MemoryEntry;
use crate::error::EngineError;

pub trait BeforeNarrationAgent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    /// On `Ok` the entries are never empty, and they carry the ranges the engine set:
    /// together they cover the Messages of the input with no gap.
    fn run(
        &self,
        input: &StoryMemoryHookInput,
        call: &CallSettings,
    ) -> Result<Vec<MemoryEntry>, EngineError>;
}

pub trait AfterNarrationAgent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn run(
        &self,
        input: &HookInput,
        settings: &AgentSettings,
    ) -> Result<QuantifierResult, EngineError>;
}

pub trait EndOfJobAgent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<OfferedSet, EngineError>;
}
