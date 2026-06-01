/// Database row for `characters` table (NpcCard).
pub struct DbCharacter {
    pub id: i64,
    pub key: String,
    pub world_id: i64,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub example_dialogue: String,
    pub summary: Option<String>,
    pub profile_image: Option<String>,
    pub headshot_image: Option<String>,
    pub inventory: String,     // JSON: Vec<String>
    pub triggers: String,      // JSON: Vec<Trigger>
    pub relationships: String, // JSON: Vec<Relationship>
    pub created_at: String,
    pub updated_at: String,
}

impl DbCharacter {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbCharacter {
            id: row.get(0)?,
            key: row.get(1)?,
            world_id: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
            personality: row.get(5)?,
            scenario: row.get(6)?,
            example_dialogue: row.get(7)?,
            summary: row.get(8)?,
            profile_image: row.get(9)?,
            headshot_image: row.get(10)?,
            inventory: row.get(11)?,
            triggers: row.get(12)?,
            relationships: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    }
}
