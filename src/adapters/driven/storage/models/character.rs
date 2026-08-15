//! [DOC: docs/diataxis/reference/storage.md]
//! Character database model

use crate::error::EngineError;
use crate::domain::model::character::{CharacterSheet, NpcCard, Relationship};
use crate::domain::model::trigger::Trigger;

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

    pub(crate) fn to_card(&self) -> Result<NpcCard, EngineError> {
        let inventory: Vec<String> = serde_json::from_str(&self.inventory)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize inventory: {e}")))?;
        let triggers: Vec<Trigger> = serde_json::from_str(&self.triggers)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize triggers: {e}")))?;
        let relationships: Vec<Relationship> = serde_json::from_str(&self.relationships)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize relationships: {e}")))?;

        Ok(NpcCard {
            id: self.key.clone(),
            sheet: CharacterSheet {
                name: self.name.clone(),
                description: self.description.clone(),
                personality: self.personality.clone(),
                scenario: self.scenario.clone(),
                example_dialogue: self.example_dialogue.clone(),
                summary: self.summary.clone().filter(|s| !s.is_empty()),
                profile_image: self.profile_image.clone().filter(|s| !s.is_empty()),
                headshot_image: self.headshot_image.clone().filter(|s| !s.is_empty()),
            },
            inventory,
            triggers,
            relationships,
        })
    }
}
