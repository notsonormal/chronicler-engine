//! [DOC: docs/diataxis/reference/storage.md]
//! Persona database model

use crate::error::EngineError;
use crate::domain::model::character::{CharacterSheet, PersonaCard};

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

    pub(crate) fn to_card(&self) -> Result<PersonaCard, EngineError> {
        let inventory: Vec<String> = serde_json::from_str(&self.inventory)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize inventory: {e}")))?;

        Ok(PersonaCard {
            key: self.key.clone(),
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
        })
    }
}
