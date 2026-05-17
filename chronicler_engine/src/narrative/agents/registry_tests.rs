use crate::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, ExecutionPhase,
};
use crate::model::settings::AppSettings;
use crate::narrative::agents::Agent;
use crate::narrative::agents::registry::AgentRegistry;

#[derive(Debug)]
struct MockAgent {
    name: String,
    phase: ExecutionPhase,
}

impl Agent for MockAgent {
    fn name(&self) -> &str {
        &self.name
    }
    fn phase(&self) -> ExecutionPhase {
        self.phase
    }
    fn backend_selector(&self) -> BackendSelector {
        BackendSelector::UseMain
    }
    fn execute(&self, _ctx: &AgentContext) -> crate::error::Result<AgentResult> {
        Ok(AgentResult::NoOp)
    }
}

#[test]
fn test_registry_filters_by_phase() {
    let mut registry = AgentRegistry::default();
    registry.add_agent(Box::new(MockAgent {
        name: "pre".to_string(),
        phase: ExecutionPhase::PreGeneration,
    }));
    registry.add_agent(Box::new(MockAgent {
        name: "post".to_string(),
        phase: ExecutionPhase::PostGeneration,
    }));

    let pre: Vec<_> = registry
        .agents_for_phase(ExecutionPhase::PreGeneration)
        .collect();
    let post: Vec<_> = registry
        .agents_for_phase(ExecutionPhase::PostGeneration)
        .collect();

    assert_eq!(pre.len(), 1);
    assert_eq!(post.len(), 1);
    assert_eq!(pre[0].name(), "pre");
    assert_eq!(post[0].name(), "post");
}

#[test]
fn test_registry_from_configs_rejects_unknown_type() {
    let configs = vec![AgentConfig {
        name: "bogus".to_string(),
        agent_type: "bogus".to_string(),
        enabled: true,
        backend: BackendSelector::UseMain,
        phase: ExecutionPhase::PostGeneration,
    }];
    let result = AgentRegistry::from_configs_with_storage(&configs, None, &AppSettings::default());
    assert!(result.is_err());
}

#[test]
fn test_registry_from_configs_empty_uses_defaults() {
    let registry =
        AgentRegistry::from_configs_with_storage(&[], None, &AppSettings::default()).unwrap();
    // Should contain the default quantifier agent
    let post: Vec<_> = registry
        .agents_for_phase(ExecutionPhase::PostGeneration)
        .collect();
    assert_eq!(post.len(), 1);
    assert_eq!(post[0].name(), "quantifier");
}

#[test]
fn test_agent_config_serde_roundtrip() {
    let config = AgentConfig {
        name: "test".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("my-backend".to_string()),
        phase: ExecutionPhase::PreGeneration,
    };
    let json = serde_json::to_string(&config).unwrap();
    let decoded: AgentConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, decoded);
}

#[test]
fn test_registry_with_agent() {
    let registry = AgentRegistry::with_agent(Box::new(MockAgent {
        name: "only".to_string(),
        phase: ExecutionPhase::PreGeneration,
    }));
    assert!(!registry.is_empty());
    let pre: Vec<_> = registry
        .agents_for_phase(ExecutionPhase::PreGeneration)
        .collect();
    assert_eq!(pre.len(), 1);
    assert_eq!(pre[0].name(), "only");
}

#[test]
fn test_registry_is_empty() {
    let empty = AgentRegistry::default();
    assert!(empty.is_empty());

    let mut non_empty = AgentRegistry::default();
    non_empty.add_agent(Box::new(MockAgent {
        name: "x".to_string(),
        phase: ExecutionPhase::PostGeneration,
    }));
    assert!(!non_empty.is_empty());
}

#[test]
fn test_registry_from_configs_disabled_skipped() {
    let configs = vec![AgentConfig {
        name: "disabled".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: false,
        backend: BackendSelector::UseMain,
        phase: ExecutionPhase::PostGeneration,
    }];
    let registry =
        AgentRegistry::from_configs_with_storage(&configs, None, &AppSettings::default()).unwrap();
    assert!(registry.is_empty());
}

#[test]
fn test_registry_from_configs_narrator_agent() {
    let configs = vec![AgentConfig {
        name: "narrator".to_string(),
        agent_type: "narrator".to_string(),
        enabled: true,
        backend: BackendSelector::UseMain,
        phase: ExecutionPhase::PreGeneration,
    }];
    let registry =
        AgentRegistry::from_configs_with_storage(&configs, None, &AppSettings::default()).unwrap();
    let pre: Vec<_> = registry
        .agents_for_phase(ExecutionPhase::PreGeneration)
        .collect();
    assert_eq!(pre.len(), 1);
    assert_eq!(pre[0].name(), "narrator");
}

#[test]
fn test_narrator_agent_returns_noop() {
    use crate::narrative::agents::registry::NarratorAgent;
    use crate::test_support::fixtures::TestGameState;

    let agent = NarratorAgent::new("test_narrator".to_string());
    assert_eq!(agent.name(), "test_narrator");
    assert_eq!(agent.phase(), ExecutionPhase::PreGeneration);
    assert_eq!(agent.backend_selector(), BackendSelector::UseMain);

    // NarratorAgent returns NoOp for any context
    let state = TestGameState::in_room("test_room");
    let ctx = AgentContext {
        state: &state,
        main_response: None,
        player_input: "hello",
        current_room: None,
    };
    let result = agent.execute(&ctx).unwrap();
    assert_eq!(result, AgentResult::NoOp);
}
