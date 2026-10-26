//! [DOC: docs/diataxis/reference/storage.md]
//! The change kinds the server counts, and one snapshot of their numbers

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::domain::model::game::GameId;

/// A kind of data that the server counts for each Game.
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameKind {
    Story,
    Options,
    BusyState,
}

impl GameKind {
    pub const ALL: [GameKind; 3] = [Self::Story, Self::Options, Self::BusyState];

    /// The field name of the kind in the answer of `GET /changes`.
    pub const fn token(self) -> &'static str {
        match self {
            Self::Story => "story",
            Self::Options => "options",
            Self::BusyState => "busy",
        }
    }
}

/// A kind of data that the server counts for the whole engine.
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    Games,
    Worlds,
    Settings,
    PromptPresets,
    LlmMessages,
}

impl EngineKind {
    pub const ALL: [EngineKind; 5] = [
        Self::Games,
        Self::Worlds,
        Self::Settings,
        Self::PromptPresets,
        Self::LlmMessages,
    ];

    /// The field name of the kind in the answer of `GET /changes`.
    pub const fn token(self) -> &'static str {
        match self {
            Self::Games => "games",
            Self::Worlds => "worlds",
            Self::Settings => "settings",
            Self::PromptPresets => "presets",
            Self::LlmMessages => "llm",
        }
    }
}

/// One kind of data that the server counts changes of. A Game kind names its Game when
/// it is raised; an engine kind names none.
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Game(GameKind),
    Engine(EngineKind),
}

impl ChangeKind {
    pub const STORY: Self = Self::Game(GameKind::Story);
    pub const OPTIONS: Self = Self::Game(GameKind::Options);
    pub const BUSY_STATE: Self = Self::Game(GameKind::BusyState);
    pub const GAMES: Self = Self::Engine(EngineKind::Games);
    pub const WORLDS: Self = Self::Engine(EngineKind::Worlds);
    pub const SETTINGS: Self = Self::Engine(EngineKind::Settings);
    pub const PROMPT_PRESETS: Self = Self::Engine(EngineKind::PromptPresets);
    pub const LLM_MESSAGES: Self = Self::Engine(EngineKind::LlmMessages);

    /// Every kind, the Game kinds first.
    pub const ALL: [ChangeKind; 8] = [
        Self::STORY,
        Self::OPTIONS,
        Self::BUSY_STATE,
        Self::GAMES,
        Self::WORLDS,
        Self::SETTINGS,
        Self::PROMPT_PRESETS,
        Self::LLM_MESSAGES,
    ];

    pub const fn token(self) -> &'static str {
        match self {
            Self::Game(kind) => kind.token(),
            Self::Engine(kind) => kind.token(),
        }
    }
}

impl std::fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.token())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameVersions {
    pub story: u64,
    pub options: u64,
    pub busy_state: u64,
}

impl GameVersions {
    pub fn number(&self, kind: GameKind) -> u64 {
        match kind {
            GameKind::Story => self.story,
            GameKind::Options => self.options,
            GameKind::BusyState => self.busy_state,
        }
    }
}

/// The active Game of a snapshot and its numbers: a snapshot holds both or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveGameVersions {
    pub id: GameId,
    pub numbers: GameVersions,
}

/// The numbers of one moment. A number only goes up, and a client compares two
/// snapshots by value. The answer of `GET /changes` is this snapshot: one flat JSON
/// object, with the field name of each kind from its `token`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeVersions {
    /// One id for each server process: a restart changes it.
    pub server: String,
    pub game: Option<ActiveGameVersions>,
    pub games: u64,
    pub worlds: u64,
    pub settings: u64,
    pub prompt_presets: u64,
    pub llm_messages: u64,
    /// One number for the whole engine: every raise of any number raises it.
    pub revision: u64,
}

impl ChangeVersions {
    /// `<server>:<revision>`: the stamp that an action answer carries and a client
    /// compares by value.
    pub fn stamp(&self) -> String {
        format!("{}:{}", self.server, self.revision)
    }

    pub fn engine_number(&self, kind: EngineKind) -> u64 {
        match kind {
            EngineKind::Games => self.games,
            EngineKind::Worlds => self.worlds,
            EngineKind::Settings => self.settings,
            EngineKind::PromptPresets => self.prompt_presets,
            EngineKind::LlmMessages => self.llm_messages,
        }
    }

    /// `None` for a Game kind when the snapshot names no Game.
    pub fn number(&self, kind: ChangeKind) -> Option<u64> {
        match kind {
            ChangeKind::Game(kind) => self.game.map(|game| game.numbers.number(kind)),
            ChangeKind::Engine(kind) => Some(self.engine_number(kind)),
        }
    }
}

impl Serialize for ChangeVersions {
    /// With no Game the answer has `"game": null` and no Game number.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut answer = serializer.serialize_map(None)?;
        answer.serialize_entry("server", &self.server)?;
        answer.serialize_entry("game", &self.game.map(|game| game.id.0))?;
        if let Some(game) = self.game {
            for kind in GameKind::ALL {
                answer.serialize_entry(kind.token(), &game.numbers.number(kind))?;
            }
        }
        for kind in EngineKind::ALL {
            answer.serialize_entry(kind.token(), &self.engine_number(kind))?;
        }
        answer.serialize_entry("revision", &self.revision)?;
        answer.end()
    }
}
