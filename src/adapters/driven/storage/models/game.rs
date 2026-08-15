//! [DOC: docs/diataxis/reference/storage.md]
//! Game database model

use crate::error::EngineError;
use crate::domain::model::game::Game;
use crate::adapters::driven::storage::utils::parse_datetime;

pub struct DbGame {
    pub id: i64,
    pub world_name: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub world_key: String,
    pub persona_key: String,
    pub persona_name: String,
}

impl DbGame {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbGame {
            id: row.get(0)?,
            world_name: row.get(1)?,
            name: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            world_key: row.get(5)?,
            persona_key: row.get(6)?,
            persona_name: row.get(7)?,
        })
    }

    pub(crate) fn to_game(&self) -> Result<Game, EngineError> {
        Ok(Game {
            id: self.id as u64,
            world_name: self.world_name.clone(),
            world_key: self.world_key.clone(),
            persona_key: self.persona_key.clone(),
            persona_name: self.persona_name.clone(),
            name: self.name.clone(),
            created_at: parse_datetime(&self.created_at, "created_at")?,
            updated_at: parse_datetime(&self.updated_at, "updated_at")?,
        })
    }
}
