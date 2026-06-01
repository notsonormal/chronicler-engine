/// Database row for `worlds` table.
pub struct DbWorld {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: String, // JSON: Vec<String>
    pub starting_room_id: String,
    pub scenarios: String, // JSON: Vec<StartingScenario>
    pub default_scenario_id: Option<String>,
    pub default_room_image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Database row for `maps` table.
pub struct DbMap {
    pub id: i64,
    pub world_id: i64,
    pub map_data: String, // JSON: full MapDef
    pub created_at: String,
    pub updated_at: String,
}
