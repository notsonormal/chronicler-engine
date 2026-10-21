//! [DOC: docs/diataxis/reference/storage.md]
//! Shared helper of the story records: the check that a write names a Message of its Game

use rusqlite::Connection;

use crate::domain::model::game::GameId;
use crate::error::{DatabaseContext, EngineError};

/// A write that names another Game's Message is a programmer error, so the unit
/// refuses it instead of writing that Game's record.
pub(crate) fn require_message_of_game(
    conn: &Connection,
    message: u64,
    game: GameId,
) -> Result<(), EngineError> {
    let found: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE id = ?1 AND game_id = ?2",
            rusqlite::params![message as i64, game.0 as i64],
            |row| row.get(0),
        )
        .db_context("Failed to find the message")?;
    if found == 0 {
        return Err(EngineError::MessageNotFound(message));
    }
    Ok(())
}
