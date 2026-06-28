//! [DOC: docs/system/storage.md]
//! Character storage backend operations

use chrono::Utc;

use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::storage::backend::{Backend, CharacterSeed, Storage};
use crate::storage::models::character::DbCharacter;

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
                rows.iter().map(character_from_db).collect()
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
                    Ok(db_char) => Ok(Some(character_from_db(&db_char)?)),
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

fn character_from_db(db: &DbCharacter) -> Result<NpcCard, EngineError> {
    use crate::model::character::CharacterSheet;
    let inventory: Vec<String> = serde_json::from_str(&db.inventory)
        .map_err(|e| EngineError::Parse(format!("Failed to deserialize inventory: {e}")))?;
    let triggers: Vec<crate::model::trigger::Trigger> = serde_json::from_str(&db.triggers)
        .map_err(|e| EngineError::Parse(format!("Failed to deserialize triggers: {e}")))?;
    let relationships: Vec<crate::model::character::Relationship> =
        serde_json::from_str(&db.relationships)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize relationships: {e}")))?;

    Ok(NpcCard {
        id: db.key.clone(),
        sheet: CharacterSheet {
            name: db.name.clone(),
            description: db.description.clone(),
            personality: db.personality.clone(),
            scenario: db.scenario.clone(),
            example_dialogue: db.example_dialogue.clone(),
            summary: db.summary.clone().filter(|s| !s.is_empty()),
            profile_image: db.profile_image.clone().filter(|s| !s.is_empty()),
            headshot_image: db.headshot_image.clone().filter(|s| !s.is_empty()),
        },
        inventory,
        triggers,
        relationships,
    })
}
