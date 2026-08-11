//! [DOC: docs/diataxis/reference/storage.md]
//! Character storage backend operations

use chrono::Utc;

use crate::error::EngineError;
use crate::domain::model::character::NpcCard;
use crate::adapters::driven::storage::backend::{Backend, CharacterSeed, Storage};
use crate::adapters::driven::storage::models::character::DbCharacter;

impl Storage {
    pub fn list_characters(&self, world_id: i64) -> Result<Vec<NpcCard>, EngineError> {
        self.with_backend_mut("list_characters", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, world_id, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, triggers, relationships, created_at, updated_at FROM characters WHERE world_id = ?",
                )?;
                let rows = stmt
                    .query_map([world_id], DbCharacter::from_row)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(EngineError::Database)?;
                rows.iter().map(DbCharacter::to_card).collect()
            }
            Backend::InMemory(data) => Ok(data.characters.iter().filter(|c| c.world_id == world_id).map(|c| c.card.clone()).collect()),
        })
    }

    pub fn get_character(&self, world_id: i64, key: &str) -> Result<Option<NpcCard>, EngineError> {
        self.with_backend_mut("get_character", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, world_id, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, triggers, relationships, created_at, updated_at FROM characters WHERE world_id = ? AND key = ?",
                )?;
                let result = stmt.query_row(rusqlite::params![world_id, key], DbCharacter::from_row);
                match result {
                    Ok(db_char) => Ok(Some(db_char.to_card()?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Database(e))
                }
            }
            Backend::InMemory(data) => Ok(data.characters.iter().find(|c| c.world_id == world_id && c.card.id == key).map(|c| c.card.clone())),
        })
    }

    pub fn seed_character(&self, world_id: i64, card: &NpcCard) -> Result<(), EngineError> {
        self.with_backend_mut("seed_character", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();
                let inventory_json = serde_json::to_string(&card.inventory)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize inventory: {e}")))?;
                let triggers_json = serde_json::to_string(&card.triggers)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize triggers: {e}")))?;
                let relationships_json = serde_json::to_string(&card.relationships)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize relationships: {e}")))?;

                conn.execute(
                    "INSERT OR IGNORE INTO characters (key, world_id, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, triggers, relationships, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        card.id,
                        world_id,
                        card.sheet.name,
                        card.sheet.description,
                        card.sheet.personality,
                        card.sheet.scenario,
                        card.sheet.example_dialogue,
                        card.sheet.summary.as_deref().filter(|s| !s.is_empty()),
                        card.sheet.profile_image.as_deref().filter(|s| !s.is_empty()),
                        card.sheet.headshot_image.as_deref().filter(|s| !s.is_empty()),
                        inventory_json,
                        triggers_json,
                        relationships_json,
                        now,
                        now,
                    ],
                )?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if !data.characters.iter().any(|c| c.world_id == world_id && c.card.id == card.id) {
                    data.characters.push(CharacterSeed {
                        world_id,
                        card: card.clone(),
                    });
                }
                Ok(())
            }
        })
    }
}

impl DbCharacter {
    fn to_card(&self) -> Result<NpcCard, EngineError> {
        use crate::domain::model::character::CharacterSheet;
        let inventory: Vec<String> = serde_json::from_str(&self.inventory)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize inventory: {e}")))?;
        let triggers: Vec<crate::domain::model::trigger::Trigger> =
            serde_json::from_str(&self.triggers)
                .map_err(|e| EngineError::Parse(format!("Failed to deserialize triggers: {e}")))?;
        let relationships: Vec<crate::domain::model::character::Relationship> =
            serde_json::from_str(&self.relationships).map_err(|e| {
                EngineError::Parse(format!("Failed to deserialize relationships: {e}"))
            })?;

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
