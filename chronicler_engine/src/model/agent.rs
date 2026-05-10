use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    PreGeneration,
    #[default]
    PostGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum BackendSelector {
    #[default]
    UseMain,
    UseNamed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: String,
    pub enabled: bool,
    #[serde(default)]
    pub backend: BackendSelector,
    #[serde(default)]
    pub phase: ExecutionPhase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatePatch {
    Scene {
        npc_ids: Vec<String>,
        movement_destination: Option<String>,
        confidence: Confidence,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentResult {
    PromptDirective(String),
    StatePatch(StatePatch),
    NoOp,
}

pub struct AgentContext<'a> {
    pub state: &'a crate::model::state::GameState,
    pub main_response: Option<&'a str>,
    pub player_input: &'a str,
}
