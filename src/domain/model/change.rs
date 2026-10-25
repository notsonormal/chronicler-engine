//! [DOC: docs/diataxis/reference/storage.md]
//! The change kinds the server counts, and one snapshot of their numbers

use serde::Serialize;

/// One kind of data that the server counts changes of. The Game kinds count per Game;
/// the others count for the whole engine.
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Story,
    Options,
    BusyState,
    Games,
    Worlds,
    Settings,
    PromptPresets,
    LlmMessages,
}

/// The numbers of the kinds that count per Game.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct GameVersions {
    pub story: u64,
    pub options: u64,
    #[serde(rename = "busy")]
    pub busy_state: u64,
}

/// The numbers of one moment. A number only goes up, and a client compares two
/// snapshots by value. The answer of `GET /changes` is this snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeVersions {
    /// One id for each server process: a restart changes it.
    pub server: String,
    /// The Game the per-Game numbers belong to.
    pub game: Option<u64>,
    /// Present together with `game`.
    #[serde(flatten)]
    pub of_game: Option<GameVersions>,
    pub games: u64,
    pub worlds: u64,
    pub settings: u64,
    #[serde(rename = "presets")]
    pub prompt_presets: u64,
    #[serde(rename = "llm")]
    pub llm_messages: u64,
}
