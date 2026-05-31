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
        match (self, other) {
            (
                StatePatch::Scene {
                    npc_ids: mut ids_a,
                    movement_destination: dest_a,
                    confidence: conf_a,
                },
                StatePatch::Scene {
                    npc_ids: ids_b,
                    movement_destination: dest_b,
                    confidence: conf_b,
                },
            ) => {
                let ids_b_unique: Vec<_> =
                    ids_b.into_iter().filter(|id| !ids_a.contains(id)).collect();
                ids_a.extend(ids_b_unique);

                let destination = match dest_a {
                    Some(ref d) => {
                        if let Some(ref db) = dest_b {
                            tracing::warn!(
                                "Movement destination conflict: {d} vs {db}, keeping first",
                            );
                        }
                        Some(d.clone())
                    }
                    None => dest_b,
                };

                let confidence = match (conf_a, conf_b) {
                    (Confidence::High, c) => c,
                    (c, Confidence::High) => c,
                    (Confidence::Medium, c) => c,
                    (c, Confidence::Medium) => c,
                    (Confidence::Low, Confidence::Low) => Confidence::Low,
                };

                StatePatch::Scene {
                    npc_ids: ids_a,
                    movement_destination: destination,
                    confidence,
                }
            }
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
    pub current_room: Option<&'a crate::model::map::Room>,
}
