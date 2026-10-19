//! Builds `WiredApp` instances for integration tests.
#![allow(clippy::expect_used)]

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::agents::AgentRunner;
use crate::application::generation::job_tasks::JobTasks;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::narration::Narrator;
use crate::application::pipeline::job::JobDeps;
use crate::application::prompting::PromptAssembler;
use crate::domain::model::agent::AgentConfig;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::utils::settings_defaults;

/// Mirrors the production system presets' macro-bearing writing style, so
/// posture macros resolve in recorded prompts.
const FIXTURE_WRITING_STYLE: &str = "{{narrative_perspective}}-person limited perspective, focused on the player character.\n{{narrative_tense}} tense narrative prose.";

pub fn seed_default_preset(storage: &Storage) {
    storage
        .save_preset(&PromptPreset {
            id: "system_default".to_string(),
            name: "Default Test System".to_string(),
            role: Some("You are a test narrator.".to_string()),
            instructions: None,
            writing_style: Some(FIXTURE_WRITING_STYLE.to_string()),
            output_format: None,
            allowed_modes: vec![NarratorMode::Novel],
            is_default: true,
            preset_type: PresetType::System,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for default preset");

    // The default Interactive Fiction bundle selects this id; without it an
    // IF-mode game built on this fixture fails preset resolution.
    storage
        .save_preset(&PromptPreset {
            id: "system_if_default".to_string(),
            name: "Test Interactive Fiction".to_string(),
            role: Some("You are a test interactive fiction narrator.".to_string()),
            instructions: None,
            writing_style: Some(FIXTURE_WRITING_STYLE.to_string()),
            output_format: None,
            allowed_modes: vec![NarratorMode::InteractiveFiction],
            is_default: true,
            preset_type: PresetType::System,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for IF default preset");

    seed_default_impersonate_preset(storage);
}

pub fn seed_default_impersonate_preset(storage: &Storage) {
    storage
        .save_preset(&PromptPreset {
            id: "impersonate_default".to_string(),
            name: "Default Test Impersonate".to_string(),
            role: Some("You are writing as {{user}}. {{persona_description}}".to_string()),
            instructions: Some("Write only as {{user}}.".to_string()),
            writing_style: Some("First person as {{user}}.".to_string()),
            output_format: Some("Write {{user}}'s next message.".to_string()),
            allowed_modes: settings_defaults::default_allowed_modes(),
            is_default: true,
            preset_type: PresetType::Impersonate,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for default impersonate preset");
}

pub fn configured_runner(recorder: Arc<LlmCallRecorder>) -> AgentRunner {
    runner_from_configs(&AgentConfig::defaults(), recorder)
}

/// Mirrors an `[agents]` section with no entry for that type.
pub fn configured_runner_without(recorder: Arc<LlmCallRecorder>, agent_type: &str) -> AgentRunner {
    let configs: Vec<AgentConfig> = AgentConfig::defaults()
        .into_iter()
        .filter(|config| config.agent_type != agent_type)
        .collect();
    runner_from_configs(&configs, recorder)
}

fn runner_from_configs(configs: &[AgentConfig], recorder: Arc<LlmCallRecorder>) -> AgentRunner {
    AgentRunner::from_configs(configs, Arc::clone(&recorder), recorder)
        // arch-lint: allow(no-unwrap-expect) reason="test wiring fails on an unknown agent config"
        .expect("configured Agents build from the default configs")
}

/// The Job's collaborators for a test: the narrator calls `recorder`, and the
/// Agents are `agent_runner`.
pub fn make_test_job_deps(
    storage: Arc<Storage>,
    recorder: Arc<LlmCallRecorder>,
    agent_runner: AgentRunner,
) -> JobDeps {
    seed_default_preset(&storage);
    JobDeps {
        narrator: Arc::new(Narrator::new(PromptAssembler, recorder)),
        agent_runner: Arc::new(agent_runner),
        storage,
        tasks: Arc::new(JobTasks::default()),
    }
}
