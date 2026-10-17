//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent hook traits

use crate::domain::model::agent::{AgentSettings, AnalysisOutput, HookInput, OfferedSet};
use crate::error::EngineError;

pub trait AnalysisAgent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn run(
        &self,
        input: &HookInput,
        settings: &AgentSettings,
    ) -> Result<AnalysisOutput, EngineError>;
}

pub trait OptionsAgent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<OfferedSet, EngineError>;
}
