//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Runtime agent lookup and lifecycle

use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::domain::model::agent::{AgentConfig, ExecutionPhase};
use crate::domain::model::settings::AppSettings;
use crate::application::agents::Agent;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::adapters::driven::storage::Storage;

#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: Vec<Box<dyn Agent>>,
}

impl AgentRegistry {
    pub fn from_configs_with_storage(
        configs: &[AgentConfig],
        quantifier_recorder: Arc<LlmCallRecorder>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Result<Self, EngineError> {
        let mut registry = Self::default();

        // If no agent configs exist, inject defaults for backward compatibility.
        // This ensures existing settings.toml files without an [agents] section
        // still get the quantifier enabled.
        let default_configs = AgentConfig::defaults();
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
                    Arc::clone(&quantifier_recorder),
                    preset_storage.clone(),
                    Arc::clone(&settings),
                )?),
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
