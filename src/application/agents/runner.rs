//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent runner: the Agents of each hook, built from their configuration

use std::sync::Arc;

use crate::application::agents::options::agent::OptionsGeneratorAgent;
use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::application::agents::trait_def::{AnalysisAgent, OptionsAgent};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::domain::model::agent::{AgentConfig, AgentSettings, AnalysisOutput, HookInput, StatePatch};
use crate::domain::model::quantifier::QuantifierResult;
use crate::error::EngineError;

#[derive(Debug, Default)]
pub struct AgentRunner {
    analysis: Vec<Box<dyn AnalysisAgent>>,
    options: Option<Box<dyn OptionsAgent>>,
}

impl AgentRunner {
    /// Builds the configured Agents. An empty config keeps the defaults, so a
    /// settings file without an `[agents]` section still runs them.
    pub fn from_configs(
        configs: &[AgentConfig],
        analysis_recorder: Arc<LlmCallRecorder>,
        options_recorder: Arc<LlmCallRecorder>,
    ) -> Result<Self, EngineError> {
        let default_configs = AgentConfig::defaults();
        let effective_configs = if configs.is_empty() {
            &default_configs[..]
        } else {
            configs
        };

        let mut runner = Self::default();
        for config in effective_configs {
            if !config.enabled {
                continue;
            }
            match config.agent_type.as_str() {
                "quantifier" => runner.analysis.push(Box::new(QuantifierAgent::from_config(
                    config,
                    Arc::clone(&analysis_recorder),
                )?)),
                "options" => {
                    runner.options = Some(Box::new(OptionsGeneratorAgent::from_config(
                        config,
                        Arc::clone(&options_recorder),
                    )?));
                }
                other => {
                    return Err(EngineError::Config(format!("Unknown agent type: {other}")));
                }
            }
        }
        Ok(runner)
    }

    /// A failed Agent is logged and contributes nothing: the Job continues with
    /// the patches of the others.
    pub fn analysis(&self, input: &HookInput, settings: &AgentSettings) -> QuantifierResult {
        let patches: Vec<StatePatch> = self
            .analysis
            .iter()
            .filter_map(|agent| match agent.run(input, settings) {
                Ok(AnalysisOutput(patch)) => Some(patch),
                Err(e) => {
                    tracing::warn!("Agent {} failed: {e}", agent.name());
                    None
                }
            })
            .collect();

        let mut result = QuantifierResult::default();
        if let Some(merged) = patches.into_iter().reduce(StatePatch::merge) {
            let StatePatch {
                npc_ids,
                movement_destination,
                confidence,
            } = merged;
            result.npcs.npc_ids = npc_ids;
            result.movement.destination = movement_destination;
            result.npcs.confidence = confidence.into();
        }
        result
    }

    /// The Options hook. `Ok(None)` says the runner holds no Options Agent, so
    /// the caller reports that Agent as unavailable; a failed Agent is logged
    /// and reported as the error it is.
    pub fn options(
        &self,
        input: &HookInput,
        settings: &AgentSettings,
    ) -> Result<Option<Vec<String>>, EngineError> {
        let Some(agent) = self.options.as_ref() else {
            return Ok(None);
        };
        match agent.run(input, settings) {
            Ok(offered) => Ok(Some(offered.0)),
            Err(e) => {
                tracing::warn!("Agent {} failed: {e}", agent.name());
                Err(e)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.analysis.is_empty() && self.options.is_none()
    }
}
