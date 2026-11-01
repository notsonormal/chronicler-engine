use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::application::agents::runner::AgentRunner;
use crate::domain::model::agent::{AgentConfig, AgentSettings, HookInput, WorldView};
use crate::domain::model::quantifier::QuantifierResult;
use crate::test_support::fixtures::{TestCharacter, TestMap, TestPersona};
use crate::test_support::make_test_recorder;

fn mock_recorder() -> Arc<crate::application::llm_recorder::LlmCallRecorder> {
    make_test_recorder(Arc::new(MockBackend::new()))
}

fn config(agent_type: &str, enabled: bool) -> AgentConfig {
    AgentConfig {
        name: agent_type.to_string(),
        agent_type: agent_type.to_string(),
        enabled,
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

/// A Job never builds an input with no text, because it always passes the saved text. So only
/// this call reaches the `Err` arm of `after_narration`: the Quantifier fails on it before
/// any model call.
fn after_narration_with_no_text(runner: &AgentRunner) -> QuantifierResult {
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let present = vec![
        TestCharacter::named("carla", "Carla"),
        TestCharacter::named("anna", "Anna"),
    ];
    let characters = HashMap::new();
    let world = WorldView {
        room: map.get_room_by_id("start"),
        map: &map,
        persona: &persona,
        characters: &characters,
        present: &present,
    };
    let input = HookInput {
        text: None,
        story: &[],
        world: &world,
    };
    runner.after_narration(&input, &AgentSettings::default())
}

fn kept_result() -> QuantifierResult {
    QuantifierResult {
        present: vec!["carla".to_string(), "anna".to_string()],
        destination: None,
    }
}

#[test]
fn test_after_narration_keeps_the_state_when_the_agent_fails() {
    let runner = AgentRunner::from_configs(
        &[config("quantifier", true)],
        mock_recorder(),
        mock_recorder(),
    )
    .expect("the runner builds");

    assert_eq!(after_narration_with_no_text(&runner), kept_result());
}
