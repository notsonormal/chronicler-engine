//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent definitions and behavior types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::state::message_types::{MessageEntry, MessageType, StoryRow};
use crate::domain::model::state::trigger_context::EventRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Writes the turn's narration.
    Narrator,
    /// Evaluates the turn and patches narrative state.
    Quantifier,
    /// Offers the pickable next actions.
    Options,
    /// Fires the configured trigger conditions.
    Trigger,
}

impl Role {
    pub const ALL: [Role; 4] = [
        Role::Narrator,
        Role::Quantifier,
        Role::Options,
        Role::Trigger,
    ];

    pub const fn agent_name(self) -> &'static str {
        match self {
            Role::Narrator => "narrator",
            Role::Quantifier => "quantifier",
            Role::Options => "options",
            Role::Trigger => "trigger",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Hook {
    /// Runs after the narrator wrote the text: the patch the game rules apply.
    #[default]
    Analysis,
    /// Runs once per Job, after its last generation: the offered set.
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A player Action or a Retry of one.
    Narration,
    /// The player dictating their own character's turn.
    Impersonate,
    /// A Retry that regenerates the player's own message.
    UserRegen,
    /// A Trigger's continuation narration.
    Event,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoryEntry {
    pub message_type: MessageType,
    pub text: String,
}

impl From<&MessageEntry> for StoryEntry {
    fn from(entry: &MessageEntry) -> Self {
        Self {
            message_type: entry.message_type.clone(),
            text: entry.text.clone(),
        }
    }
}

impl From<&StoryRow> for StoryEntry {
    fn from(row: &StoryRow) -> Self {
        Self {
            message_type: row.message_type.clone(),
            text: row.text.clone(),
        }
    }
}

pub const STORY_WINDOW: usize = 4;

pub struct WorldView<'a> {
    /// `None` when the state names a room the map does not know.
    pub room: Option<&'a Room>,
    pub map: &'a MapDef,
    pub persona: &'a PersonaCard,
    pub characters: &'a HashMap<String, NpcCard>,
    pub present: &'a [NpcCard],
}

/// The hook step owns every read and write an Agent needs.
pub struct HookInput<'a> {
    pub kind: Kind,
    /// The text the Job saved, for a hook that runs after the narrator call.
    pub text: Option<&'a str>,
    pub player_text: &'a str,
    pub story: &'a [StoryEntry],
    pub world: &'a WorldView<'a>,
    pub event: Option<&'a EventRecord>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgentSettings {
    pub preset: Option<String>,
    pub max_tokens: Option<u32>,
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
    pub hook: Hook,
}

impl AgentConfig {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "quantifier".to_string(),
                agent_type: "quantifier".to_string(),
                enabled: true,
                backend: BackendSelector::UseNamed("quantifier".to_string()),
                hook: Hook::Analysis,
            },
            Self {
                name: "options".to_string(),
                agent_type: "options".to_string(),
                enabled: true,
                backend: BackendSelector::UseNamed("options".to_string()),
                hook: Hook::Options,
            },
        ]
    }
}

impl StatePatch {
    /// A second movement destination is a contradiction between two Agents, so the
    /// first wins.
    pub fn merge(self, other: StatePatch) -> StatePatch {
        let mut npc_ids = self.npc_ids;
        let new_ids: Vec<String> = other
            .npc_ids
            .into_iter()
            .filter(|id| !npc_ids.contains(id))
            .collect();
        npc_ids.extend(new_ids);

        let movement_destination = match (self.movement_destination, other.movement_destination) {
            (Some(first), Some(second)) => {
                tracing::warn!("Movement destination conflict: {first} vs {second}, keeping first");
                Some(first)
            }
            (Some(first), None) => Some(first),
            (None, destination) => destination,
        };

        StatePatch {
            npc_ids,
            movement_destination,
            confidence: self.confidence.min(other.confidence),
        }
    }
}

/// [TRIVIAL_ENUM]
///
/// Ordered from the least to the most confident, so `min` of two analyses keeps
/// the one that is least sure of itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatePatch {
    pub npc_ids: Vec<String>,
    pub movement_destination: Option<String>,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisOutput(pub StatePatch);

#[derive(Clone, Debug, PartialEq)]
pub struct OfferedSet(pub Vec<String>);
