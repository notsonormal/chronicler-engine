//! Unit tests for the Role agent names and the Agent configuration

use crate::domain::model::agent::{AgentConfig, StoryMemoryHookInput, Role};
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::test_support::fixtures::default_story_row;

#[test]
fn test_role_agent_names_are_the_recorded_names() {
    // A forensic LlmMessage row stores one of these names, so they are a
    // storage contract that a variant rename must not move.
    assert_eq!(
        Role::ALL.map(Role::agent_name),
        ["narrator", "quantifier", "options", "trigger", "memory"]
    );
}

#[test]
fn test_agent_config_serde_roundtrip() {
    let config = AgentConfig {
        name: "test".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
    };
    let json = serde_json::to_string(&config).expect("the config serializes");
    let decoded: AgentConfig = serde_json::from_str(&json).expect("the config deserializes");
    assert_eq!(config, decoded);
}

#[test]
fn test_agent_config_stored_with_a_backend_key_still_loads() {
    let json = r#"{"name":"quantifier","agent_type":"quantifier","enabled":true,
        "backend":{"type":"use_named","value":"quantifier"}}"#;

    let config: AgentConfig = serde_json::from_str(json).expect("a stored agents column loads");

    assert_eq!(config.name, "quantifier");
    assert!(config.enabled);
}

#[test]
fn test_memory_hook_input_needs_a_leaving_message() {
    assert!(StoryMemoryHookInput::new(None, &[], "Julian").is_none());
}

#[test]
fn test_memory_hook_input_names_its_oldest_and_newest_message() {
    let leaving = [12, 13, 15].map(|id| StoryRow {
        id,
        ..default_story_row(MessageType::Narration)
    });

    let input = StoryMemoryHookInput::new(None, &leaving, "Julian").expect("Messages leave");

    assert_eq!((input.oldest(), input.newest()), (12, 15));
    assert_eq!(input.leaving().len(), 3);
}
