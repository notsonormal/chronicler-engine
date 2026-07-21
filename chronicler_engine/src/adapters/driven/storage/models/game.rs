//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]
//! Game database model

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
}
