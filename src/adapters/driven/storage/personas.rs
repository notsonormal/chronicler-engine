//! [DOC: docs/diataxis/reference/storage.md]
//! Persona storage operations

use chrono::Utc;

use crate::error::EngineError;
use crate::domain::model::character::PersonaCard;
use crate::adapters::driven::storage::{Backend, Storage};
use crate::adapters::driven::storage::in_memory_data::PersonaCardWithKey;
use crate::adapters::driven::storage::models::persona::DbPersona;

impl Storage {
    pub fn list_personas(&self) -> Result<Vec<PersonaCard>, EngineError> {
        self.with_backend_mut("list_personas", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare("SELECT id, key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, created_at, updated_at FROM personas")?;
                let rows = stmt
                    .query_map([], DbPersona::from_row)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(EngineError::Database)?;
                rows.iter().map(DbPersona::to_card).collect()
            }
            Backend::InMemory(data) => Ok(data.personas.iter().map(|p| p.card.clone()).collect()),
        })
    }

    pub fn get_persona(&self, key: &str) -> Result<Option<PersonaCard>, EngineError> {
        self.with_backend_mut("get_persona", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, created_at, updated_at FROM personas WHERE key = ?",
                )?;
                let result = stmt.query_row([key], DbPersona::from_row);
                match result {
                    Ok(db_persona) => Ok(Some(db_persona.to_card()?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Database(e))
                }
            }
            Backend::InMemory(data) => Ok(data.personas.iter().find(|p| p.key == key).map(|p| p.card.clone())),
        })
    }

    /// Required-read of a persona row by key. Absence becomes
    /// [`EngineError::PersonaNotFound`]. Catalogue / fallback callers should
    /// stay on [`Storage::get_persona`](Self::get_persona).
    pub fn require_persona(&self, key: &str) -> Result<PersonaCard, EngineError> {
        self.get_persona(key)?
            .ok_or_else(|| EngineError::PersonaNotFound(key.to_string()))
    }

    pub fn seed_persona(&self, key: &str, card: &PersonaCard) -> Result<(), EngineError> {
        self.with_backend_mut("seed_persona", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();
                let inventory_json = serde_json::to_string(&card.inventory)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize inventory: {e}")))?;

                conn.execute(
                    "INSERT OR IGNORE INTO personas (key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        key,
                        card.sheet.name,
                        card.sheet.description,
                        card.sheet.personality,
                        card.sheet.scenario,
                        card.sheet.example_dialogue,
                        card.sheet.summary.as_deref().filter(|s| !s.is_empty()),
                        card.sheet.profile_image.as_deref().filter(|s| !s.is_empty()),
                        card.sheet.headshot_image.as_deref().filter(|s| !s.is_empty()),
                        inventory_json,
                        now,
                        now,
                    ],
                )?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if !data.personas.iter().any(|p| p.key == key) {
                    data.personas.push(PersonaCardWithKey {
                        key: key.to_string(),
                        card: card.clone(),
                    });
                }
                Ok(())
            }
        })
    }
}
