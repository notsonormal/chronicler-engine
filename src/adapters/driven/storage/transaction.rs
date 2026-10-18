//! [DOC: docs/diataxis/reference/storage.md]
//! The transaction: one unit of record writes that lands whole

use crate::adapters::driven::storage::core::WriteOverrides;
use crate::adapters::driven::storage::{Backend, Storage};
use crate::domain::model::game::{GameId, NewGame};
use crate::error::{EngineError, InternalError};

/// One record write of a `Tx`, keyed by a type so a failure injection stays
/// compiler-checked.
/// [TRIVIAL_ENUM]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TxWrite {
    InsertGame,
    DeleteGame,
}

/// One unit of record writes, held open by `Storage::transaction`; it lands
/// whole or rolls back.
///
/// The closure makes no `Storage` call, which would wait on the backend mutex
/// the unit already holds.
pub struct Tx<'a> {
    conn: rusqlite::Transaction<'a>,
    writes: WriteOverrides,
}

impl<'a> Tx<'a> {
    pub(crate) fn new(conn: rusqlite::Transaction<'a>, writes: WriteOverrides) -> Self {
        Self { conn, writes }
    }

    pub fn insert_game(&mut self, request: &NewGame) -> Result<GameId, EngineError> {
        self.writes.fail_if_injected(TxWrite::InsertGame)?;
        Self::write_game_row(&self.conn, request)
    }

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

    /// The Game row, and the rows the schema cascades from it.
    pub fn delete_game(&mut self, game_id: GameId) -> Result<(), EngineError> {
        self.writes.fail_if_injected(TxWrite::DeleteGame)?;
        self.conn
            .execute(
                "DELETE FROM games WHERE id = ?1",
                rusqlite::params![game_id.0 as i64],
            )
            .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
        Ok(())
    }

    fn commit(self) -> Result<(), EngineError> {
        self.conn
            .commit()
            .map_err(|e| EngineError::Config(format!("Failed to commit transaction: {e}")))
    }
}

impl Storage {
    /// The unit holds the backend mutex for its whole run; an `Err` or a panic
    /// from `f` rolls it back.
    pub fn transaction<F, T>(&self, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Tx<'_>) -> Result<T, EngineError>,
    {
        self.with_backend_writes("transaction", |backend, writes| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let unit = conn.unchecked_transaction().map_err(|e| {
                    EngineError::Config(format!("Failed to begin transaction: {e}"))
                })?;
                let mut tx = Tx::new(unit, writes);
                let value = f(&mut tx)?;
                tx.commit()?;
                Ok(value)
            }
            Backend::InMemory(_) => Err(EngineError::Internal(InternalError::new(
                "transaction: the in-memory backend holds no unit; tier 1 runs on SQLite",
            ))),
        })
    }

    /// The Game row goes first, so its id reaches the writes that follow it in
    /// the same unit.
    pub fn transaction_new_game<F, T>(&self, request: &NewGame, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(GameId, &mut Tx<'_>) -> Result<T, EngineError>,
    {
        self.transaction(|tx| {
            let game_id = tx.insert_game(request)?;
            f(game_id, tx)
        })
    }
}
