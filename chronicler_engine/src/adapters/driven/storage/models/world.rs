//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]
//! Database row structs for world and map tables

pub struct DbWorld {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String, // JSON: Vec<String>
    pub scenarios: String,    // JSON: Vec<StartingScenario>
    pub default_scenario_id: Option<String>,
    pub default_room_image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl DbWorld {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbWorld {
            id: row.get(0)?,
            key: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            global_rules: row.get(4)?,
            scenarios: row.get(5)?,
            default_scenario_id: row.get(6)?,
            default_room_image: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}

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
