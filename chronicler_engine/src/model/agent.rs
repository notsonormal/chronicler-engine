//! [DOC: docs/system/agent_system.md]
//! Agent definitions and behavior types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    #[default]
    PreGeneration,
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

impl StatePatch {
    /// Merge another StatePatch into this one.
    ///
    /// Merge semantics:
    /// - `npc_ids`: union of unique IDs, preserving first-seen order
    /// - `movement_destination`: keep first non-None, warn on conflict
    /// - `confidence`: take minimum (most conservative)
    pub fn merge(self, other: StatePatch) -> StatePatch {
        let ids_b_unique: Vec<_> = other
            .npc_ids
            .into_iter()
            .filter(|id| !self.npc_ids.contains(id))
            .collect();
        let mut npc_ids = self.npc_ids;
        npc_ids.extend(ids_b_unique);

        let movement_destination = match self.movement_destination {
            Some(ref d) => {
                if let Some(ref db) = other.movement_destination {
                    tracing::warn!("Movement destination conflict: {d} vs {db}, keeping first",);
                }
                Some(d.clone())
            }
            None => other.movement_destination,
        };

        let confidence = match (self.confidence, other.confidence) {
            (Confidence::High, c) => c,
            (c, Confidence::High) => c,
            (Confidence::Medium, c) => c,
            (c, Confidence::Medium) => c,
            (Confidence::Low, Confidence::Low) => Confidence::Low,
        };

        StatePatch {
            npc_ids,
            movement_destination,
            confidence,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatePatch {
    pub npc_ids: Vec<String>,
    pub movement_destination: Option<String>,
    pub confidence: Confidence,
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
    pub current_room: Option<&'a crate::model::map::Room>,
}
