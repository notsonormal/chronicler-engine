//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent definitions and behavior types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::state::generation_status::Step;
use crate::domain::model::state::message_types::StoryRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Writes a generation's Narration.
    Narrator,
    /// Reads a generation's new text and gives the Quantifier result.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A player Action or a Retry of one.
    Narration,
    /// The player dictating their own Persona's Message.
    Impersonate,
    /// A Retry that regenerates the player's own message.
    UserRegen,
    /// A Trigger's continuation narration.
    Event,
}

impl Kind {
    /// The step whose name the status shows while this generation runs.
    pub fn step(self) -> Step {
        match self {
            Self::Event => Step::GeneratingEvent,
            _ => Step::Narrating,
        }
    }

    /// The role whose health a generation's model call feeds.
    pub fn role(self) -> Role {
        match self {
            Self::Event => Role::Trigger,
            _ => Role::Narrator,
        }
    }

    /// Whether a generation of this kind saves a Narration: the text save empties
    /// the dock and the Options run may follow it. An Input save does neither.
    pub fn saves_a_narration(self) -> bool {
        matches!(self, Self::Narration | Self::Event)
    }
}

/// How many of the newest Messages an Agent reads as its history.
pub const AGENT_HISTORY_WINDOW: usize = 4;

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
    /// The text the Job saved, for a hook that runs after the narrator call.
    pub text: Option<&'a str>,
    pub story: &'a [StoryRow],
    pub world: &'a WorldView<'a>,
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
}

impl AgentConfig {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "quantifier".to_string(),
                agent_type: "quantifier".to_string(),
                enabled: true,
                backend: BackendSelector::UseNamed("quantifier".to_string()),
            },
            Self {
                name: "options".to_string(),
                agent_type: "options".to_string(),
                enabled: true,
                backend: BackendSelector::UseNamed("options".to_string()),
            },
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OfferedSet(pub Vec<String>);
