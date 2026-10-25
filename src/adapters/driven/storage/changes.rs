//! [DOC: docs/diataxis/reference/storage.md]
//! The change counters: one number for each kind of data, raised after the data is committed

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::adapters::driven::storage::core::StorageCall;
use crate::adapters::driven::storage::transaction::TxWrite;
use crate::domain::model::change::{ChangeKind, ChangeVersions, GameVersions};
use crate::domain::model::game::GameId;

/// One lock holds every number, so a snapshot reads them together.
#[derive(Default)]
struct Numbers {
    games: u64,
    worlds: u64,
    settings: u64,
    prompt_presets: u64,
    llm_messages: u64,
    per_game: HashMap<GameId, GameVersions>,
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
            game: game.map(|game| game.0),
            of_game: game.map(|game| numbers.per_game.get(&game).copied().unwrap_or_default()),
            games: numbers.games,
            worlds: numbers.worlds,
            settings: numbers.settings,
            prompt_presets: numbers.prompt_presets,
            llm_messages: numbers.llm_messages,
        }
    }

    pub fn raise(&self, game: Option<GameId>, kinds: &[ChangeKind]) {
        let mut numbers = self.numbers();
        for &kind in kinds {
            match kind {
                ChangeKind::Games => numbers.games += 1,
                ChangeKind::Worlds => numbers.worlds += 1,
                ChangeKind::Settings => numbers.settings += 1,
                ChangeKind::PromptPresets => numbers.prompt_presets += 1,
                ChangeKind::LlmMessages => numbers.llm_messages += 1,
                ChangeKind::Story | ChangeKind::Options | ChangeKind::BusyState => {
                    let Some(game) = game else {
                        debug_assert!(
                            false,
                            "{kind:?} counts per Game, and its raise names no Game"
                        );
                        continue;
                    };
                    let of_game = numbers.per_game.entry(game).or_default();
                    match kind {
                        ChangeKind::Story => of_game.story += 1,
                        ChangeKind::Options => of_game.options += 1,
                        _ => of_game.busy_state += 1,
                    }
                }
            }
        }
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

    /// A read raises none, and a transaction raises its own kinds at its commit.
    #[deny(clippy::wildcard_enum_match_arm)]
    pub(crate) fn kinds_of_call(call: StorageCall) -> &'static [ChangeKind] {
        match call {
            StorageCall::CreateWorld
            | StorageCall::DeleteWorld
            | StorageCall::SeedCharacter
            | StorageCall::SeedWorld
            | StorageCall::UpdateWorld => &[ChangeKind::Worlds],
            StorageCall::DeletePreset | StorageCall::SavePreset => &[ChangeKind::PromptPresets],
            StorageCall::SaveLlmMessage => &[ChangeKind::LlmMessages],
            StorageCall::SaveSettings | StorageCall::SeedSettings | StorageCall::UpdateSettings => {
                &[ChangeKind::Settings]
            }
            StorageCall::SeedGameRow
            | StorageCall::TouchGameUpdatedAt
            | StorageCall::UpdateGameDisplayName => &[ChangeKind::Games],
            StorageCall::CountGamesUsingPreset
            | StorageCall::DocksGet
            | StorageCall::GetCharacter
            | StorageCall::GetGame
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
                &[ChangeKind::Story, ChangeKind::Games]
            }
            TxWrite::InsertSwipe
            | TxWrite::SetSwipeText
            | TxWrite::SetSwipeState
            | TxWrite::SetActiveSwipe => &[ChangeKind::Story],
            TxWrite::PutDock | TxWrite::PutOptionsFailure | TxWrite::ClearDock => {
                &[ChangeKind::Options]
            }
            TxWrite::SetGameFailure => &[ChangeKind::BusyState],
            TxWrite::InsertGame | TxWrite::DeleteGame | TxWrite::SetGameConfig => {
                &[ChangeKind::Games]
            }
        }
    }
}

impl Default for ChangeCounters {
    fn default() -> Self {
        Self::new()
    }
}
