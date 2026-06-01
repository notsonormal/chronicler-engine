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
