//! [DOC: docs/diataxis/reference/storage.md]
//! The dock record reads: the offered set and the notice of one Game

use crate::adapters::driven::storage::utils::{from_json, no_in_memory};
use crate::adapters::driven::storage::{Backend, Storage};
use crate::domain::model::dock::{DockRecord, Notice};
use crate::domain::model::game::GameId;
use crate::error::EngineError;

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
            .with_backend_mut("docks_get", |backend| match backend {
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
                            return Err(EngineError::Config(format!(
                                "Failed to read the dock record: {other}"
                            )));
                        }
                    };

                    let offered: Vec<String> = serde_json::from_str(&row.0).map_err(|e| {
                        EngineError::Config(format!("Failed to parse the offered set: {e}"))
                    })?;
                    let notice: Option<Notice> = from_json("dock notice", row.1.as_deref())?;
                    Ok(Some(DockRecord { offered, notice }))
                }
                Backend::InMemory(_) => Err(no_in_memory("docks_get")),
            })
    }
}
