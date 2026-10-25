//! [DOC: docs/diataxis/reference/storage.md]
//! Character storage backend operations

use chrono::Utc;

use crate::error::{DatabaseContext, EngineError};
use crate::domain::model::character::CharacterCard;
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::adapters::driven::storage::models::character::DbCharacter;

impl Storage {
    pub fn list_characters(&self, world_id: i64) -> Result<Vec<CharacterCard>, EngineError> {
        self.with_backend_mut(StorageCall::ListCharacters, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, world_id, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, triggers, relationships, created_at, updated_at FROM characters WHERE world_id = ?",
                ).db_context("Failed to prepare the characters query")?;
                let rows = stmt
                    .query_map([world_id], DbCharacter::from_row)
                    .db_context("Failed to query the characters")?
                    .collect::<Result<Vec<_>, _>>()
                    .db_context("Failed to read a character row")?;
                rows.iter().map(DbCharacter::to_card).collect()
            }
        })
    }

    pub fn get_character(
        &self,
        world_id: i64,
        key: &str,
    ) -> Result<Option<CharacterCard>, EngineError> {
        self.with_backend_mut(StorageCall::GetCharacter, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, world_id, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, triggers, relationships, created_at, updated_at FROM characters WHERE world_id = ? AND key = ?",
                ).db_context("Failed to prepare the character read")?;
                let result = stmt.query_row(rusqlite::params![world_id, key], DbCharacter::from_row);
                match result {
                    Ok(db_char) => Ok(Some(db_char.to_card()?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e).db_context("Failed to read the character"),
                }
            }
        })
    }

    pub fn seed_character(&self, world_id: i64, card: &CharacterCard) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::SeedCharacter, |backend| match backend {
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
                )
                .db_context("Failed to seed the character")?;
                Ok(())
            }
        })
    }
}
