//! [DOC: docs/diataxis/reference/storage.md]
//! Database row struct for the `maps` table

pub struct DbMap {
    pub id: i64,
    pub world_id: i64,
    pub map_data: String, // JSON: full MapDef
    pub created_at: String,
    pub updated_at: String,
}

impl DbMap {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbMap {
            id: row.get(0)?,
            world_id: row.get(1)?,
            map_data: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}
