//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent runner: the Agents of each hook, built from their configuration

use std::sync::Arc;

use crate::application::agents::options::agent::OptionsGeneratorAgent;
use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::application::agents::trait_def::{AfterNarrationAgent, EndOfJobAgent};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::domain::model::agent::{AgentConfig, AgentSettings, HookInput};
use crate::domain::model::quantifier::QuantifierResult;
use crate::error::EngineError;

#[derive(Debug, Default)]
pub struct AgentRunner {
    after_narration: Option<Box<dyn AfterNarrationAgent>>,
    end_of_job: Option<Box<dyn EndOfJobAgent>>,
}

impl AgentRunner {
    /// Builds the configured Agents. An empty config keeps the defaults, so a
    /// settings file without an `[agents]` section still runs them.
    pub fn from_configs(
        configs: &[AgentConfig],
        quantifier_recorder: Arc<LlmCallRecorder>,
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
                "quantifier" => {
                    let agent =
                        QuantifierAgent::from_config(config, Arc::clone(&quantifier_recorder))?;
                    if runner.after_narration.replace(Box::new(agent)).is_some() {
                        tracing::warn!(
                            "A second after-narration Agent config replaces the first: {}",
                            config.name
                        );
                    }
                }
                "options" => {
                    let agent =
                        OptionsGeneratorAgent::from_config(config, Arc::clone(&options_recorder))?;
                    if runner.end_of_job.replace(Box::new(agent)).is_some() {
                        tracing::warn!(
                            "A second end-of-Job Agent config replaces the first: {}",
                            config.name
                        );
                    }
                }
                other => {
                    return Err(EngineError::Config(format!("Unknown agent type: {other}")));
                }
            }
        }
        Ok(runner)
    }

    /// A failed Agent is logged and gives the empty Quantifier result, so the
    /// Job continues with no Character present and no destination.
    pub fn after_narration(&self, input: &HookInput, settings: &AgentSettings) -> QuantifierResult {
        let Some(agent) = self.after_narration.as_ref() else {
            return QuantifierResult::default();
        };
        agent.run(input, settings).unwrap_or_else(|e| {
            tracing::warn!("Agent {} failed: {e}", agent.name());
            QuantifierResult::default()
        })
    }

    /// `Ok(None)` says the runner holds no end-of-Job Agent, so the caller reports
    /// that Agent as unavailable; a failed Agent is logged and reported as the
    /// error it is.
    pub fn end_of_job(
        &self,
        input: &HookInput,
        settings: &AgentSettings,
    ) -> Result<Option<Vec<String>>, EngineError> {
        let Some(agent) = self.end_of_job.as_ref() else {
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
        self.after_narration.is_none() && self.end_of_job.is_none()
    }
}
