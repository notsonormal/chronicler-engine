//! [DOC: docs/system/storage.md]
//! Persona database model

/// Database row for `personas` table (PersonaCard).
pub struct DbPersona {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub example_dialogue: String,
    pub summary: Option<String>,
    pub profile_image: Option<String>,
    pub headshot_image: Option<String>,
    pub inventory: String, // JSON: Vec<String>
    pub created_at: String,
    pub updated_at: String,
}

impl DbPersona {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbPersona {
            id: row.get(0)?,
            key: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            personality: row.get(4)?,
            scenario: row.get(5)?,
            example_dialogue: row.get(6)?,
            summary: row.get(7)?,
            profile_image: row.get(8)?,
            headshot_image: row.get(9)?,
            inventory: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }
}
