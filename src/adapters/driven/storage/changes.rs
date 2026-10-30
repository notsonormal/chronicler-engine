//! [DOC: docs/diataxis/reference/storage.md]
//! The change counters: one number for each kind of data, raised after the data is committed

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::adapters::driven::storage::core::StorageCall;
use crate::adapters::driven::storage::transaction::TxWrite;
use crate::domain::model::change::{
    ActiveGameVersions, ChangeKind, ChangeVersions, EngineKind, GameKind, GameVersions,
};
use crate::domain::model::game::GameId;

/// One lock holds every number, so a snapshot reads them together. Each of the two
/// inner raise functions adds to `revision`, so no raise path skips it.
#[derive(Default)]
struct Numbers {
    games: u64,
    worlds: u64,
    settings: u64,
    prompt_presets: u64,
    llm_messages: u64,
    revision: u64,
    per_game: HashMap<GameId, GameVersions>,
}

impl Numbers {
    fn raise_game(&mut self, game: GameId, kind: GameKind) {
        self.revision += 1;
        let of_game = self.per_game.entry(game).or_default();
        match kind {
            GameKind::Story => of_game.story += 1,
            GameKind::Options => of_game.options += 1,
            GameKind::BusyState => of_game.busy_state += 1,
        }
    }

    fn raise_engine(&mut self, kind: EngineKind) {
        self.revision += 1;
        match kind {
            EngineKind::Games => self.games += 1,
            EngineKind::Worlds => self.worlds += 1,
            EngineKind::Settings => self.settings += 1,
            EngineKind::PromptPresets => self.prompt_presets += 1,
            EngineKind::LlmMessages => self.llm_messages += 1,
        }
    }
}

/// Memory only. `Storage` owns one, and a restart makes a new server id, so a
/// client that holds old numbers sees the restart and loads everything again.
pub struct ChangeCounters {
    server: String,
    numbers: Mutex<Numbers>,
}

impl ChangeCounters {
    pub fn new() -> Self {
        Self {
            server: uuid::Uuid::new_v4().to_string(),
            numbers: Mutex::new(Numbers::default()),
        }
    }

    /// A Game with no raise yet reads as zeros, and the read makes no entry.
    pub fn versions(&self, game: Option<GameId>) -> ChangeVersions {
        let numbers = self.numbers();
        ChangeVersions {
            server: self.server.clone(),
            game: game.map(|id| ActiveGameVersions {
                id,
                numbers: numbers.per_game.get(&id).copied().unwrap_or_default(),
            }),
            games: numbers.games,
            worlds: numbers.worlds,
            settings: numbers.settings,
            prompt_presets: numbers.prompt_presets,
            llm_messages: numbers.llm_messages,
            revision: numbers.revision,
        }
    }

    /// Raises the Game kinds of one Game.
    pub fn raise_game(&self, game: GameId, kinds: &[GameKind]) {
        let mut numbers = self.numbers();
        for &kind in kinds {
            numbers.raise_game(game, kind);
        }
        Self::log(Some(game), kinds);
    }

    /// Raises engine-wide kinds.
    pub fn raise_engine(&self, kinds: &[EngineKind]) {
        let mut numbers = self.numbers();
        for &kind in kinds {
            numbers.raise_engine(kind);
        }
        Self::log(None, kinds);
    }

    /// Raises the kinds of one committed unit, all under one lock: a Game kind goes to
    /// the Game the unit serves.
    pub(crate) fn raise_unit(&self, game: GameId, kinds: &[ChangeKind]) {
        let mut numbers = self.numbers();
        for &kind in kinds {
            match kind {
                ChangeKind::Game(kind) => numbers.raise_game(game, kind),
                ChangeKind::Engine(kind) => numbers.raise_engine(kind),
            }
        }
        Self::log(Some(game), kinds);
    }

    fn log(game: Option<GameId>, kinds: &[impl std::fmt::Debug]) {
        if !kinds.is_empty() {
            tracing::debug!(?game, ?kinds, "change raised");
        }
    }

    /// A deleted Game keeps no numbers. The Game's own deletion ends its holder, so no
    /// later raise names it.
    pub fn forget_game(&self, game: GameId) {
        self.numbers().per_game.remove(&game);
    }

    fn numbers(&self) -> MutexGuard<'_, Numbers> {
        self.numbers.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The engine kinds a successful call raises. A read raises none. `SeedPersona`
    /// writes, but no kind counts Personas, so it raises none. A transaction raises its
    /// own kinds at its commit.
    #[deny(clippy::wildcard_enum_match_arm)]
    pub(crate) fn kinds_of_call(call: StorageCall) -> &'static [EngineKind] {
        match call {
            StorageCall::CreateWorld
            | StorageCall::DeleteWorld
            | StorageCall::SeedCharacter
            | StorageCall::SeedWorld
            | StorageCall::UpdateWorld => &[EngineKind::Worlds],
            StorageCall::DeletePreset | StorageCall::SavePreset => &[EngineKind::PromptPresets],
            StorageCall::SaveLlmMessage => &[EngineKind::LlmMessages],
            StorageCall::SaveSettings | StorageCall::SeedSettings | StorageCall::UpdateSettings => {
                &[EngineKind::Settings]
            }
            StorageCall::SeedGameRow
            | StorageCall::TouchGameUpdatedAt
            | StorageCall::UpdateGameDisplayName => &[EngineKind::Games],
            StorageCall::CountGamesUsingPreset
            | StorageCall::CountSavedGames
            | StorageCall::DocksGet
            | StorageCall::GetCharacter
            | StorageCall::GetGame
            | StorageCall::GetLlmMessage
            | StorageCall::GetPersona
            | StorageCall::GetPreset
            | StorageCall::GetSettings
            | StorageCall::GetWorld
            | StorageCall::GetWorldById
            | StorageCall::ListCharacters
            | StorageCall::ListGames
            | StorageCall::ListLatestLlmMessages
            | StorageCall::ListPersonas
            | StorageCall::ListPresets
            | StorageCall::ListSavedGames
            | StorageCall::ListWorlds
            | StorageCall::NewestGameOfWorld
            | StorageCall::SeedPersona
            | StorageCall::StoriesActiveSwipe
            | StorageCall::StoriesEntries
            | StorageCall::StoriesHasText
            | StorageCall::StoriesMessages
            | StorageCall::StoriesSwipeState
            | StorageCall::StoriesTail
            | StorageCall::StoriesWindow
            | StorageCall::Transaction => &[],
        }
    }

    /// A new or a removed Message also raises the Games list, which orders the Games by
    /// their newest Message.
    #[deny(clippy::wildcard_enum_match_arm)]
    pub(crate) fn kinds_of_write(write: TxWrite) -> &'static [ChangeKind] {
        match write {
            TxWrite::InsertMessage | TxWrite::DeleteMessage => {
                &[ChangeKind::STORY, ChangeKind::GAMES]
            }
            TxWrite::InsertSwipe
            | TxWrite::SetSwipeText
            | TxWrite::SetSwipeState
            | TxWrite::SetActiveSwipe => &[ChangeKind::STORY],
            TxWrite::PutDock | TxWrite::PutOptionsFailure | TxWrite::ClearDock => {
                &[ChangeKind::OPTIONS]
            }
            TxWrite::SetGameFailure => &[ChangeKind::BUSY_STATE],
            TxWrite::InsertGame | TxWrite::DeleteGame | TxWrite::SetGameConfig => {
                &[ChangeKind::GAMES]
            }
        }
    }
}

impl Default for ChangeCounters {
    fn default() -> Self {
        Self::new()
    }
}
