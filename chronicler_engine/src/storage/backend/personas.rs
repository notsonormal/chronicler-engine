//! [DOC: docs/system/storage.md]
//! Persona storage operations

use chrono::Utc;

use crate::error::EngineError;
use crate::model::character::PlayerCard;
use crate::storage::backend::{empty_to_none, Backend, Operation, PlayerCardWithKey, Storage};
use crate::storage::models::persona::DbPersona;

impl Storage {
    pub fn list_personas(&self) -> Result<Vec<PlayerCard>, EngineError> {
        self.with_backend_mut(Operation::ListPersonas, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare("SELECT id, key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, created_at, updated_at FROM personas")?;
                let rows = stmt
                    .query_map([], DbPersona::from_row)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(EngineError::Database)?;
                rows.iter().map(persona_from_db).collect()
            }
            Backend::InMemory(data) => Ok(data.personas.iter().map(|p| p.card.clone()).collect()),
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn get_persona(&self, key: &str) -> Result<Option<PlayerCard>, EngineError> {
        self.with_backend_mut(Operation::GetPersona, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory, created_at, updated_at FROM personas WHERE key = ?",
                )?;
                let result = stmt.query_row([key], DbPersona::from_row);
                match result {
                    Ok(db_persona) => Ok(Some(persona_from_db(&db_persona)?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Database(e))
                }
            }
            Backend::InMemory(data) => Ok(data.personas.iter().find(|p| p.key == key).map(|p| p.card.clone())),
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn seed_persona(&self, key: &str, card: &PlayerCard) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::SeedPersona, |backend, _game_id| match backend {
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
                        empty_to_none(card.sheet.summary.as_deref().unwrap_or("")),
                        empty_to_none(card.sheet.profile_image.as_deref().unwrap_or("")),
                        empty_to_none(card.sheet.headshot_image.as_deref().unwrap_or("")),
                        inventory_json,
                        now,
                        now,
                    ],
                )?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if !data.personas.iter().any(|p| p.key == key) {
                    data.personas.push(PlayerCardWithKey {
                        key: key.to_string(),
                        card: card.clone(),
                    });
                }
                Ok(())
            }
            Backend::Test { .. } => unimplemented!(),
        })
    }
}

fn persona_from_db(db: &DbPersona) -> Result<PlayerCard, EngineError> {
    use crate::model::character::CharacterSheet;
    let inventory: Vec<String> = serde_json::from_str(&db.inventory)
        .map_err(|e| EngineError::Parse(format!("Failed to deserialize inventory: {e}")))?;

    Ok(PlayerCard {
        key: db.key.clone(),
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
    })
}
