//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent definitions and behavior types

use serde::{Deserialize, Serialize};

use crate::domain::model::state::game_state::GameState;

/// The registry's dispatch axis — agents declare the pipeline position
/// where they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    /// Agents that run before narration is generated.
    #[default]
    PreGeneration,
    /// Agents the generic post-narration loop runs, whose `StatePatch`
    /// output merges into the quantifier result.
    PostGeneration,
    /// Agents dispatched only at gated call sites — the turn-end offered-set
    /// rewrite and the `/options` entry path. The generic PostGeneration
    /// merge loop never runs them.
    OptionsGeneration,
}

/// [TRIVIAL_ENUM]
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

impl AgentConfig {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "quantifier".to_string(),
                agent_type: "quantifier".to_string(),
                enabled: true,
                backend: BackendSelector::UseNamed("quantifier".to_string()),
                phase: ExecutionPhase::PostGeneration,
            },
            Self {
                name: "options".to_string(),
                agent_type: "options".to_string(),
                enabled: true,
                backend: BackendSelector::UseNamed("options".to_string()),
                phase: ExecutionPhase::OptionsGeneration,
            },
        ]
    }
}

impl StatePatch {
    /// Union `npc_ids`; keep first non-None `movement_destination` (warn on conflict); take minimum `confidence`.
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

/// [TRIVIAL_ENUM]
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

/// What an agent produced for one pipeline run.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentResult {
    /// A directive the agent wants folded into the next narration prompt;
    /// the current merge loop carries and ignores it.
    PromptDirective(String),
    /// Structured state mutations (scene NPCs, movement, confidence) merged
    /// into the quantifier result.
    StatePatch(StatePatch),
    /// A generated pickable-option set; the options dispatch sites write it
    /// to the game's current options state.
    Options(Vec<String>),
    /// The agent ran and has nothing to contribute this turn.
    NoOp,
}

pub struct AgentContext<'a> {
    pub state: &'a GameState,
    pub main_response: Option<&'a str>,
    pub player_input: &'a str,
    pub current_room: Option<&'a crate::domain::model::map::Room>,
    pub map: &'a crate::domain::model::map::MapDef,
    pub persona: &'a crate::domain::model::character::PersonaCard,
    pub npcs: &'a std::collections::HashMap<String, crate::domain::model::character::NpcCard>,
}
