/// Database row for `personas` table (PlayerCard).
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
