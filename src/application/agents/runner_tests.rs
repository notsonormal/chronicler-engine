use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::application::agents::runner::AgentRunner;
use crate::domain::model::agent::{AgentConfig, BackendSelector};
use crate::test_support::make_test_recorder;

fn mock_recorder() -> Arc<crate::application::llm_recorder::LlmCallRecorder> {
    make_test_recorder(Arc::new(MockBackend::new()))
}

fn config(agent_type: &str, enabled: bool) -> AgentConfig {
    AgentConfig {
        name: agent_type.to_string(),
        agent_type: agent_type.to_string(),
        enabled,
        backend: BackendSelector::UseMain,
    }
}

#[test]
fn test_runner_from_configs_rejects_unknown_type() {
    let configs = vec![config("bogus", true)];
    let result = AgentRunner::from_configs(&configs, mock_recorder(), mock_recorder());
    assert!(result.is_err());
}

#[test]
fn test_runner_from_configs_empty_uses_defaults() {
    let runner =
        AgentRunner::from_configs(&[], mock_recorder(), mock_recorder()).expect("defaults build");
    assert!(!runner.is_empty());
}

#[test]
fn test_runner_from_configs_disabled_skipped() {
    let configs = vec![config("quantifier", false)];
    let runner = AgentRunner::from_configs(&configs, mock_recorder(), mock_recorder())
        .expect("a disabled Agent still builds a runner");
    assert!(runner.is_empty());
}

#[test]
fn test_runner_from_configs_keeps_one_agent_per_hook() {
    let configs = vec![config("quantifier", true), config("quantifier", true)];
    let runner = AgentRunner::from_configs(&configs, mock_recorder(), mock_recorder())
        .expect("a second config of one hook replaces the first");
    assert!(!runner.is_empty());
}
