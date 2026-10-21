//! [DOC: docs/diataxis/reference/storage.md]
//! The dock record reads: the offered set and the notice of one Game

use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::domain::model::dock::DockRecord;
use crate::domain::model::game::GameId;
use crate::error::{DatabaseContext, EngineError};

pub struct DockReads<'a> {
    storage: &'a Storage,
}

impl Storage {
    pub fn docks(&self) -> DockReads<'_> {
        DockReads { storage: self }
    }
}

impl DockReads<'_> {
    pub fn get(&self, game: GameId) -> Result<Option<DockRecord>, EngineError> {
        self.storage
            .with_backend_mut(StorageCall::DocksGet, |backend| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let row = match conn.query_row(
                        "SELECT offered, notice FROM game_docks WHERE game_id = ?1",
                        rusqlite::params![game.0 as i64],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
                    ) {
                        Ok(row) => row,
                        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                        Err(other) => {
                            return Err(other).db_context("Failed to read the dock record");
                        }
                    };

                    Ok(Some(DockRecord {
                        offered: serde_json::from_str(&row.0)?,
                        notice: row.1.as_deref().map(serde_json::from_str).transpose()?,
                    }))
                }
            })
    }
}
