//! [DOC: docs/diataxis/reference/storage.md]
//! Shared helpers of the new records: the in-memory answer and the JSON columns

use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::domain::model::game::GameId;
use crate::error::{EngineError, InternalError};

/// The new records run on SQLite only: tier 1 runs on SQLite, and the in-memory
/// backend holds no record of the rebuild.
pub(crate) fn no_in_memory(operation: &'static str) -> EngineError {
    EngineError::Internal(InternalError::new(format!(
        "{operation}: the in-memory backend holds no new record; tier 1 runs on SQLite"
    )))
}

/// The JSON a column holds. `label` names the record in the error, because a
/// parse failure is what a corrupt row shows.
pub(crate) fn to_json<T: Serialize>(label: &str, value: &T) -> Result<String, EngineError> {
    serde_json::to_string(value)
        .map_err(|e| EngineError::Config(format!("Failed to serialize {label}: {e}")))
}

/// The JSON a nullable column holds, or none.
pub(crate) fn to_optional_json<T: Serialize>(
    label: &str,
    value: Option<&T>,
) -> Result<Option<String>, EngineError> {
    value.map(|value| to_json(label, value)).transpose()
}

/// The value a column holds. `label` names the record in the error, because a
/// parse failure is what a corrupt row shows.
pub(crate) fn from_json<T: DeserializeOwned>(label: &str, json: &str) -> Result<T, EngineError> {
    serde_json::from_str(json)
        .map_err(|e| EngineError::Config(format!("Failed to parse {label}: {e}")))
}

/// The value a nullable column holds, or none.
pub(crate) fn from_optional_json<T: DeserializeOwned>(
    label: &str,
    json: Option<&str>,
) -> Result<Option<T>, EngineError> {
    json.map(|json| from_json(label, json)).transpose()
}

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
        .map_err(|e| EngineError::Config(format!("Failed to find the message: {e}")))?;
    if found == 0 {
        return Err(EngineError::MessageNotFound(message));
    }
    Ok(())
}
