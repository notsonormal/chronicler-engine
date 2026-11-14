//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Agent definitions and behavior types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::{CharacterCard, PersonaCard};
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::state::generation_status::Step;
use crate::domain::model::state::message_types::StoryRow;
use crate::domain::model::story_memory::StoryMemory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Writes a generation's Narration.
    Narrator,
    /// Reads a generation's new text and gives the Quantifier result.
    Quantifier,
    /// Offers Options to pick from.
    Options,
    /// Fires the configured trigger conditions.
    Trigger,
    /// Rewrites the story memory when Messages leave the history.
    StoryMemory,
}

impl Role {
    pub const ALL: [Role; 5] = [
        Role::Narrator,
        Role::Quantifier,
        Role::Options,
        Role::Trigger,
        Role::StoryMemory,
    ];

    pub const fn agent_name(self) -> &'static str {
        match self {
            Role::Narrator => "narrator",
            Role::Quantifier => "quantifier",
            Role::Options => "options",
            Role::Trigger => "trigger",
            Role::StoryMemory => "memory",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A player Action or a Retry of one.
    Narration,
    /// The player dictating their own Persona's Message.
    Impersonate,
    /// The Retry of a typed Input: the player's own message is written again.
    InputRetry,
    /// A Trigger's continuation narration.
    Event,
}

impl Kind {
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

pub const AGENT_HISTORY_WINDOW: usize = 4;

pub struct WorldView<'a> {
    /// `None` when the state names a room the map does not know.
    pub room: Option<&'a Room>,
    pub map: &'a MapDef,
    pub persona: &'a PersonaCard,
    pub characters: &'a HashMap<String, CharacterCard>,
    pub present: &'a [CharacterCard],
}

/// The Messages that leave the history and the story memory they join. `leaving` is
/// in story order, oldest first, and never empty: with nothing leaving there is no
/// input, so the writer never runs.
pub struct StoryMemoryHookInput<'a> {
    pub previous: Option<&'a StoryMemory>,
    pub persona_name: &'a str,
    leaving: &'a [StoryRow],
    oldest: u64,
    newest: u64,
}

impl<'a> StoryMemoryHookInput<'a> {
    pub fn new(
        previous: Option<&'a StoryMemory>,
        leaving: &'a [StoryRow],
        persona_name: &'a str,
    ) -> Option<Self> {
        let oldest = leaving.first()?.id;
        let newest = leaving.last()?.id;
        Some(Self {
            previous,
            persona_name,
            leaving,
            oldest,
            newest,
        })
    }

    pub fn leaving(&self) -> &'a [StoryRow] {
        self.leaving
    }

    pub fn oldest(&self) -> u64 {
        self.oldest
    }

    /// The new story memory covers the Messages up to this one.
    pub fn newest(&self) -> u64 {
        self.newest
    }
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
    pub call: CallSettings,
    pub option_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: String,
    pub enabled: bool,
}

impl AgentConfig {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "quantifier".to_string(),
                agent_type: "quantifier".to_string(),
                enabled: true,
            },
            Self {
                name: "options".to_string(),
                agent_type: "options".to_string(),
                enabled: true,
            },
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OfferedSet(pub Vec<String>);
