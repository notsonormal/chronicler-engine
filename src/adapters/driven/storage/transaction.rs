//! [DOC: docs/diataxis/reference/storage.md]
//! The transaction: one unit of record writes that lands whole

use crate::adapters::driven::storage::core::WriteOverrides;
use crate::adapters::driven::storage::utils::require_message_of_game;
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::domain::model::dock::{DockRecord, Notice};
use crate::domain::model::game::{ConfigChange, GameId, NewGame};
use crate::domain::model::message::{Message, Swipe, SwipeAt};
use crate::domain::model::state::generation_status::GenerationFailure;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::error::{DatabaseContext, EngineError};

/// Keyed by a type, so a failure injection stays compiler-checked.
/// [TRIVIAL_ENUM]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TxWrite {
    InsertGame,
    DeleteGame,
    InsertMessage,
    SetActiveSwipe,
    DeleteMessage,
    InsertSwipe,
    SetSwipeText,
    SetSwipeState,
    SetGameFailure,
    SetGameConfig,
    PutDock,
    PutDockNotice,
    ClearDock,
}

/// The capability a unit of record writes needs. Its constructor is crate-only, so a
/// unit opens in `gate.rs` and nowhere else; a test gets its token from
/// `test_support`. A hold hands a clone to each unit it opens.
#[derive(Clone)]
pub struct TxToken(());

impl TxToken {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

/// Every write goes to the Game the `Tx` holds, so no write can reach another Game:
/// one unit serves one Game, and the create unit holds the Game it just wrote.
///
/// The closure makes no `Storage` call, which would wait on the backend mutex
/// the unit already holds.
pub struct Tx<'a> {
    conn: rusqlite::Transaction<'a>,
    writes: WriteOverrides,
    game: GameId,
}

impl<'a> Tx<'a> {
    pub(crate) fn new(
        conn: rusqlite::Transaction<'a>,
        writes: WriteOverrides,
        game: GameId,
    ) -> Self {
        Self { conn, writes, game }
    }

    /// The row lands before the `Tx` exists, so the unit's writes already know the
    /// Game they write.
    pub(crate) fn write_game_row(
        conn: &rusqlite::Connection,
        request: &NewGame,
    ) -> Result<GameId, EngineError> {
        let now = chrono::Utc::now().to_rfc3339();
        let display_name = request.display_name();
        conn.execute(
            "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, display_name, created_at, updated_at, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id, active_options_prompt_preset_id, options_always_on) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                request.world_name,
                request.world_key,
                request.persona_key,
                request.persona_name,
                request.name,
                display_name,
                &now,
                request.narrator_mode.as_str(),
                request.narrative_perspective.as_str(),
                request.narrative_tense.as_str(),
                request.system_prompt_preset_id,
                request.quantifier_prompt_preset_id,
                request.impersonate_prompt_preset_id,
                request.options_prompt_preset_id,
                request.options_always_on as i64,
            ],
        )
        .db_context("Failed to create game")?;
        Ok(GameId(conn.last_insert_rowid() as u64))
    }

    /// The Game row, with the records the schema cascades from it. A reset removes
    /// the active Game in this unit and creates the new one in its own.
    pub fn delete_game(&mut self) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::DeleteGame)?;
        self.conn
            .execute(
                "DELETE FROM games WHERE id = ?1",
                rusqlite::params![self.game.0 as i64],
            )
            .db_context("Failed to delete game")?;
        Ok(())
    }

    pub fn insert_message(&mut self, message: &Message) -> Result<u64, EngineError> {
        self.writes.fail_if_injected(TxWrite::InsertMessage)?;
        let message_type_json = serde_json::to_string(&message.message_type)?;
        self.conn
            .execute(
                "INSERT INTO messages (game_id, message_type, kind, timestamp, active_swipe_index)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    self.game.0 as i64,
                    message_type_json,
                    serde_json::to_string(&message.kind)?,
                    message.timestamp.to_rfc3339(),
                    message.active_swipe_index as i64,
                ],
            )
            .db_context("Failed to insert message")?;
        Ok(self.conn.last_insert_rowid() as u64)
    }

    pub fn set_active_swipe(&mut self, at: SwipeAt) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::SetActiveSwipe)?;
        require_message_of_game(&self.conn, at.message, self.game)?;
        self.conn
            .execute(
                "UPDATE messages SET active_swipe_index = ?1 WHERE id = ?2",
                rusqlite::params![at.swipe as i64, at.message as i64],
            )
            .db_context("Failed to set the active swipe")?;
        Ok(())
    }

    /// Deleting the Message takes the rows the schema cascades from it.
    pub fn delete_message(&mut self, message: u64) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::DeleteMessage)?;
        require_message_of_game(&self.conn, message, self.game)?;
        self.conn
            .execute(
                "DELETE FROM messages WHERE id = ?1",
                rusqlite::params![message as i64],
            )
            .db_context("Failed to delete message")?;
        Ok(())
    }

    pub fn insert_swipe(&mut self, at: SwipeAt, swipe: &Swipe) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::InsertSwipe)?;
        require_message_of_game(&self.conn, at.message, self.game)?;
        self.conn
            .execute(
                "INSERT INTO message_swipes (message_id, swipe_index, text, event_header, impersonated, guide_text, impersonate_direction, state, maker)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    at.message as i64,
                    at.swipe as i64,
                    &swipe.text,
                    swipe.event_header.as_deref(),
                    if swipe.impersonated { 1 } else { 0 },
                    swipe.guide_text.as_deref(),
                    swipe.impersonate_direction.as_deref(),
                    serde_json::to_string(&swipe.state)?,
                    serde_json::to_string(&swipe.maker)?,
                ],
            )
            .db_context("Failed to insert swipe")?;
        Ok(())
    }

    pub fn set_swipe_text(&mut self, at: SwipeAt, text: &str) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::SetSwipeText)?;
        require_message_of_game(&self.conn, at.message, self.game)?;
        self.conn
            .execute(
                "UPDATE message_swipes SET text = ?1 WHERE message_id = ?2 AND swipe_index = ?3",
                rusqlite::params![text, at.message as i64, at.swipe as i64],
            )
            .db_context("Failed to set the swipe text")?;
        Ok(())
    }

    pub fn set_swipe_state(&mut self, at: SwipeAt, state: &SwipeState) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::SetSwipeState)?;
        require_message_of_game(&self.conn, at.message, self.game)?;
        self.conn
            .execute(
                "UPDATE message_swipes SET state = ?1 WHERE message_id = ?2 AND swipe_index = ?3",
                rusqlite::params![
                    serde_json::to_string(state)?,
                    at.message as i64,
                    at.swipe as i64,
                ],
            )
            .db_context("Failed to set the swipe state")?;
        Ok(())
    }

    pub fn set_game_failure(
        &mut self,
        failure: Option<&GenerationFailure>,
    ) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::SetGameFailure)?;
        self.conn
            .execute(
                "UPDATE games SET failure = ?1 WHERE id = ?2",
                rusqlite::params![
                    failure.map(serde_json::to_string).transpose()?,
                    self.game.0 as i64
                ],
            )
            .db_context("Failed to set the failure line")?;
        Ok(())
    }

    /// Writes the columns `change` names on the Game row the `Tx` holds, and
    /// `updated_at`. Every other column stays as it is.
    pub fn set_game_config(&mut self, change: &ConfigChange) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::SetGameConfig)?;
        let now = chrono::Utc::now().to_rfc3339();
        let game = self.game.0 as i64;
        let affected = match change {
            ConfigChange::Posture { perspective, tense } => self.conn.execute(
                "UPDATE games SET narrative_perspective=?, narrative_tense=?, updated_at=? WHERE id=?",
                rusqlite::params![perspective.as_str(), tense.as_str(), &now, game],
            ),
            ConfigChange::Mode {
                mode,
                system_id,
                quantifier_id,
                impersonate_id,
                perspective,
            } => self.conn.execute(
                "UPDATE games SET narrator_mode=?, narrative_perspective=?, active_system_prompt_preset_id=?, active_quantifier_prompt_preset_id=?, active_impersonate_prompt_preset_id=?, updated_at=? WHERE id=?",
                rusqlite::params![
                    mode.as_str(),
                    perspective.as_str(),
                    system_id,
                    quantifier_id,
                    impersonate_id,
                    &now,
                    game,
                ],
            ),
            ConfigChange::Presets(selection) => self.conn.execute(
                "UPDATE games SET active_system_prompt_preset_id=?, active_quantifier_prompt_preset_id=?, active_impersonate_prompt_preset_id=?, active_options_prompt_preset_id=?, updated_at=? WHERE id=?",
                rusqlite::params![
                    selection.system_id,
                    selection.quantifier_id,
                    selection.impersonate_id,
                    selection.options_id,
                    &now,
                    game,
                ],
            ),
        }
        .db_context("Failed to update game config")?;
        if affected == 0 {
            return Err(EngineError::GameNotFound(self.game.0));
        }
        Ok(())
    }

    pub fn put_dock(&mut self, record: &DockRecord) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::PutDock)?;
        self.write_dock(record)
    }

    /// Replaces the notice and keeps the offered set the row holds.
    pub fn put_dock_notice(&mut self, notice: &Notice) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::PutDockNotice)?;
        self.conn
            .execute(
                "INSERT INTO game_docks (game_id, notice) VALUES (?1, ?2) \
                 ON CONFLICT(game_id) DO UPDATE SET notice = excluded.notice",
                rusqlite::params![self.game.0 as i64, serde_json::to_string(notice)?],
            )
            .db_context("Failed to write the dock notice")?;
        Ok(())
    }

    /// The row stays, with no offered set and no notice: a Game holds one dock
    /// record.
    pub fn clear_dock(&mut self) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::ClearDock)?;
        self.write_dock(&DockRecord::default())
    }

    fn write_dock(&mut self, record: &DockRecord) -> Result<(), EngineError> {
        let offered = serde_json::to_string(&record.offered)?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO game_docks (game_id, offered, notice) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    self.game.0 as i64,
                    offered,
                    record
                        .notice
                        .as_ref()
                        .map(serde_json::to_string)
                        .transpose()?,
                ],
            )
            .db_context("Failed to write the dock record")?;
        Ok(())
    }

    fn commit(self) -> Result<(), EngineError> {
        self.conn
            .commit()
            .db_context("Failed to commit transaction")
    }
}

/// The Game a unit serves: one that exists, or the one the unit creates first.
/// [TRIVIAL_ENUM]
enum UnitGame<'a> {
    Existing(GameId),
    New(&'a NewGame),
}

impl Storage {
    /// Holds the backend mutex for the closure's whole run; an `Err` or a panic
    /// from `f` rolls the unit back. The unit serves `game`, and it cannot write
    /// another Game's rows.
    pub fn transaction<F, T>(&self, game: GameId, _token: TxToken, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    {
        self.run_transaction(UnitGame::Existing(game), |_, tx| f(tx))
    }

    /// The create unit: the Game row goes first, so its id reaches the writes
    /// that follow it in the same unit.
    pub fn transaction_new_game<F, T>(
        &self,
        request: &NewGame,
        _token: TxToken,
        f: F,
    ) -> Result<T, EngineError>
    where
        F: FnOnce(GameId, &mut Tx<'_>) -> Result<T, EngineError>,
    {
        self.run_transaction(UnitGame::New(request), f)
    }

    fn run_transaction<F, T>(&self, unit_game: UnitGame<'_>, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(GameId, &mut Tx<'_>) -> Result<T, EngineError>,
    {
        self.with_backend_writes(StorageCall::Transaction, |backend, writes| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let unit = conn
                    .unchecked_transaction()
                    .db_context("Failed to begin transaction")?;
                let game = match unit_game {
                    UnitGame::Existing(game) => game,
                    UnitGame::New(request) => {
                        writes.fail_if_injected(TxWrite::InsertGame)?;
                        Tx::write_game_row(&unit, request)?
                    }
                };
                let mut tx = Tx::new(unit, writes, game);
                let value = f(game, &mut tx)?;
                tx.commit()?;
                Ok(value)
            }
        })
    }
}
