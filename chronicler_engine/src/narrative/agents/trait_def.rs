use crate::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase};

pub trait Agent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn phase(&self) -> ExecutionPhase;
    fn backend_selector(&self) -> BackendSelector;
    fn execute(&self, ctx: &AgentContext) -> crate::error::Result<AgentResult>;
}
