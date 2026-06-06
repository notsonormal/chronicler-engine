//! [DOC: docs/system/agent_system.md]
//! Runtime agent lookup and lifecycle

use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::model::agent::{AgentConfig, AgentResult, BackendSelector, ExecutionPhase};
use crate::model::settings::AppSettings;
use crate::narrative::agents::Agent;
use crate::narrative::agents::quantifier::QuantifierAgent;
use crate::storage::Storage;

#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: Vec<Box<dyn Agent>>,
}

impl AgentRegistry {
    pub fn from_configs(configs: &[AgentConfig]) -> Result<Self, EngineError> {
        Self::from_configs_with_storage(
            configs,
            None,
            None,
            Arc::new(RwLock::new(AppSettings::default())),
        )
    }

    pub fn from_configs_with_storage(
        configs: &[AgentConfig],
        storage: Option<Arc<Storage>>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Result<Self, EngineError> {
        let mut registry = Self::default();

        // If no agent configs exist, inject defaults for backward compatibility.
        // This ensures existing settings.toml files without an [agents] section
        // still get the quantifier enabled.
        let default_configs = crate::model::settings::default_agent_configs();
        let effective_configs = if configs.is_empty() {
            &default_configs[..]
        } else {
            configs
        };

        for config in effective_configs {
            if !config.enabled {
                continue;
            }
            let agent: Box<dyn Agent> = match config.agent_type.as_str() {
                "quantifier" => Box::new(QuantifierAgent::from_config_with_storage(
                    config,
                    storage.clone(),
                    preset_storage.clone(),
                    Arc::clone(&settings),
                )?),
                "narrator" => Box::new(NarratorAgent::new(config.name.clone())),
                other => {
                    return Err(EngineError::Config(format!("Unknown agent type: {other}")));
                }
            };
            registry.agents.push(agent);
        }
        Ok(registry)
    }

    pub fn with_agent(agent: Box<dyn Agent>) -> Self {
        Self {
            agents: vec![agent],
        }
    }

    pub fn add_agent(&mut self, agent: Box<dyn Agent>) {
        self.agents.push(agent);
    }

    pub fn agents_for_phase(&self, phase: ExecutionPhase) -> impl Iterator<Item = &dyn Agent> {
        self.agents
            .iter()
            .filter(move |a| a.phase() == phase)
            .map(|a| a.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

#[derive(Debug)]
pub struct NarratorAgent {
    name: String,
}

impl NarratorAgent {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Agent for NarratorAgent {
    fn name(&self) -> &str {
        &self.name
    }
    fn phase(&self) -> ExecutionPhase {
        ExecutionPhase::PreGeneration
    }
    fn backend_selector(&self) -> BackendSelector {
        BackendSelector::UseMain
    }
    fn execute(
        &self,
        _ctx: &crate::model::agent::AgentContext,
    ) -> Result<AgentResult, EngineError> {
        Ok(AgentResult::NoOp)
    }
}
