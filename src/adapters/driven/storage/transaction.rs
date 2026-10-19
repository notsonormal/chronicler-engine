//! [DOC: docs/diataxis/reference/storage.md]
//! The transaction: one unit of record writes that lands whole

use crate::adapters::driven::storage::core::WriteOverrides;
use crate::adapters::driven::storage::utils::{
    no_in_memory, require_message_of_game, to_json, to_optional_json,
};
use crate::adapters::driven::storage::{Backend, Storage};
use crate::domain::model::dock::DockRecord;
use crate::domain::model::game::{GameId, NewGame};
use crate::domain::model::message::{Message, Swipe, SwipeAt};
use crate::domain::model::state::generation_status::GenerationFailure;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::error::EngineError;

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
    PutDock,
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
        .map_err(|e| EngineError::Config(format!("Failed to create game: {e}")))?;
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
            .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
        Ok(())
    }

    pub fn insert_message(&mut self, message: &Message) -> Result<u64, EngineError> {
        self.writes.fail_if_injected(TxWrite::InsertMessage)?;
        let message_type_json = serde_json::to_string(&message.message_type)
            .map_err(|e| EngineError::Config(format!("Failed to serialize message type: {e}")))?;
        self.conn
            .execute(
                "INSERT INTO messages (game_id, message_type, kind, timestamp, active_swipe_index)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    self.game.0 as i64,
                    message_type_json,
                    to_json("message kind", &message.kind)?,
                    message.timestamp.to_rfc3339(),
                    message.active_swipe_index as i64,
                ],
            )
            .map_err(|e| EngineError::Config(format!("Failed to insert message: {e}")))?;
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
            .map_err(|e| EngineError::Config(format!("Failed to set the active swipe: {e}")))?;
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
            .map_err(|e| EngineError::Config(format!("Failed to delete message: {e}")))?;
        Ok(())
    }

    pub fn insert_swipe(&mut self, at: SwipeAt, swipe: &Swipe) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::InsertSwipe)?;
        require_message_of_game(&self.conn, at.message, self.game)?;
        self.conn
            .execute(
                "INSERT INTO message_swipes (message_id, swipe_index, text, room_report, event_header, impersonated, guide_text, impersonate_direction, state, maker)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    at.message as i64,
                    at.swipe as i64,
                    &swipe.text,
                    swipe.room_report.as_deref(),
                    swipe.event_header.as_deref(),
                    if swipe.impersonated { 1 } else { 0 },
                    swipe.guide_text.as_deref(),
                    swipe.impersonate_direction.as_deref(),
                    to_json("swipe state", &swipe.state)?,
                    to_json("swipe maker", &swipe.maker)?,
                ],
            )
            .map_err(|e| EngineError::Config(format!("Failed to insert swipe: {e}")))?;
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
            .map_err(|e| EngineError::Config(format!("Failed to set the swipe text: {e}")))?;
        Ok(())
    }

    pub fn set_swipe_state(
        &mut self,
        at: SwipeAt,
        state: &SwipeState,
        room_report: Option<&str>,
    ) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::SetSwipeState)?;
        require_message_of_game(&self.conn, at.message, self.game)?;
        self.conn
            .execute(
                "UPDATE message_swipes SET state = ?1, room_report = ?2 WHERE message_id = ?3 AND swipe_index = ?4",
                rusqlite::params![
                    to_json("swipe state", state)?,
                    room_report,
                    at.message as i64,
                    at.swipe as i64,
                ],
            )
            .map_err(|e| EngineError::Config(format!("Failed to set the swipe state: {e}")))?;
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
                    to_optional_json("game failure", failure)?,
                    self.game.0 as i64
                ],
            )
            .map_err(|e| EngineError::Config(format!("Failed to set the failure line: {e}")))?;
        Ok(())
    }

    pub fn put_dock(&mut self, record: &DockRecord) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::PutDock)?;
        self.write_dock(record)
    }

    /// The row stays, with no offered set and no notice: a Game holds one dock
    /// record.
    pub fn clear_dock(&mut self) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::ClearDock)?;
        self.write_dock(&DockRecord::default())
    }

    fn write_dock(&mut self, record: &DockRecord) -> Result<(), EngineError> {
        let offered = serde_json::to_string(&record.offered).map_err(|e| {
            EngineError::Config(format!("Failed to serialize the offered set: {e}"))
        })?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO game_docks (game_id, offered, notice) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    self.game.0 as i64,
                    offered,
                    to_optional_json("dock notice", record.notice.as_ref())?,
                ],
            )
            .map_err(|e| EngineError::Config(format!("Failed to write the dock record: {e}")))?;
        Ok(())
    }

    fn commit(self) -> Result<(), EngineError> {
        self.conn
            .commit()
            .map_err(|e| EngineError::Config(format!("Failed to commit transaction: {e}")))
    }
}

impl Storage {
    /// Holds the backend mutex for the closure's whole run; an `Err` or a panic
    /// from `f` rolls the unit back. The unit serves `game`, and it cannot write
    /// another Game's rows.
    pub fn transaction<F, T>(&self, game: GameId, _token: TxToken, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    {
        self.with_backend_writes("transaction", |backend, writes| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let unit = conn.unchecked_transaction().map_err(|e| {
                    EngineError::Config(format!("Failed to begin transaction: {e}"))
                })?;
                let mut tx = Tx::new(unit, writes, game);
                let value = f(&mut tx)?;
                tx.commit()?;
                Ok(value)
            }
            Backend::InMemory(_) => Err(no_in_memory("transaction")),
        })
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
        self.with_backend_writes("transaction", |backend, writes| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let unit = conn.unchecked_transaction().map_err(|e| {
                    EngineError::Config(format!("Failed to begin transaction: {e}"))
                })?;
                writes.fail_if_injected(TxWrite::InsertGame)?;
                let game_id = Tx::write_game_row(&unit, request)?;
                let mut tx = Tx::new(unit, writes, game_id);
                let value = f(game_id, &mut tx)?;
                tx.commit()?;
                Ok(value)
            }
            Backend::InMemory(_) => Err(no_in_memory("transaction")),
        })
    }
}
