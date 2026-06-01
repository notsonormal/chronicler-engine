use chrono::Utc;

use crate::error::EngineError;
use crate::model::character::PlayerCard;
use crate::storage::backend::{Backend, Operation, PlayerCardWithKey, Storage};
use crate::storage::models::persona::DbPersona;

impl Storage {
    pub fn list_personas(&self) -> Result<Vec<PlayerCard>, EngineError> {
        self.with_backend_mut(Operation::ListPersonas, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare("SELECT id, key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory FROM personas")?;
                let rows = stmt.query_map([], |row| {
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
                        created_at: String::new(),
                        updated_at: String::new(),
                    })
                })?;
                let mut personas = Vec::new();
                for row_result in rows {
                    let db_persona = row_result?;
                    personas.push(persona_from_db(&db_persona)?);
                }
                Ok(personas)
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
                    "SELECT id, key, name, description, personality, scenario, example_dialogue, summary, profile_image, headshot_image, inventory FROM personas WHERE key = ?",
                )?;
                let row = stmt.query_row([key], |row| {
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
                        created_at: String::new(),
                        updated_at: String::new(),
                    })
                });
                match row {
                    Ok(db_persona) => Ok(Some(persona_from_db(&db_persona)?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Database(e.to_string())),
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
                    [
                        key,
                        &card.sheet.name,
                        &card.sheet.description,
                        &card.sheet.personality,
                        &card.sheet.scenario,
                        &card.sheet.example_dialogue,
                        card.sheet.summary.as_deref().unwrap_or(""),
                        card.sheet.profile_image.as_deref().unwrap_or(""),
                        card.sheet.headshot_image.as_deref().unwrap_or(""),
                        &inventory_json,
                        &now,
                        &now,
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
        sheet: CharacterSheet {
            name: db.name.clone(),
            description: db.description.clone(),
            personality: db.personality.clone(),
            scenario: db.scenario.clone(),
            example_dialogue: db.example_dialogue.clone(),
            summary: if db.summary.as_deref().unwrap_or("").is_empty() {
                None
            } else {
                db.summary.clone()
            },
            profile_image: if db.profile_image.as_deref().unwrap_or("").is_empty() {
                None
            } else {
                db.profile_image.clone()
            },
            headshot_image: if db.headshot_image.as_deref().unwrap_or("").is_empty() {
                None
            } else {
                db.headshot_image.clone()
            },
        },
        inventory,
    })
}
