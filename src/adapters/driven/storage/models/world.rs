//! [DOC: docs/diataxis/reference/storage.md]
//! Database row struct for the `worlds` table

use crate::error::EngineError;
use crate::domain::model::world::WorldCard;

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

    pub(crate) fn to_card(&self) -> Result<WorldCard, EngineError> {
        Ok(WorldCard {
            key: self.key.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            global_rules: serde_json::from_str(&self.global_rules).map_err(|e| {
                EngineError::Parse(format!("Failed to deserialize global_rules: {e}"))
            })?,
            scenarios: serde_json::from_str(&self.scenarios)
                .map_err(|e| EngineError::Parse(format!("Failed to deserialize scenarios: {e}")))?,
            default_scenario_id: self.default_scenario_id.clone().filter(|s| !s.is_empty()),
            default_room_image: self.default_room_image.clone().filter(|s| !s.is_empty()),
        })
    }
}
