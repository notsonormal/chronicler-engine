//! Unit tests for the Role agent names and the Agent configuration

use crate::domain::model::agent::{AgentConfig, BackendSelector, Role};

#[test]
fn test_role_agent_names_are_the_recorded_names() {
    // A forensic LlmMessage row stores one of these names, so they are a
    // storage contract that a variant rename must not move.
    assert_eq!(
        Role::ALL.map(Role::agent_name),
        ["narrator", "quantifier", "options", "trigger"]
    );
}

#[test]
fn test_agent_config_serde_roundtrip() {
    let config = AgentConfig {
        name: "test".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("my-backend".to_string()),
    };
    let json = serde_json::to_string(&config).expect("the config serializes");
    let decoded: AgentConfig = serde_json::from_str(&json).expect("the config deserializes");
    assert_eq!(config, decoded);
}
