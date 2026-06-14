//! [DOC: docs/system/storage.md]
//! World storage backend operations

use chrono::Utc;

use crate::error::EngineError;
use crate::model::map::MapDef;
use crate::model::world::WorldCard;
use crate::storage::backend::{Backend, Operation, Storage, InMemoryWorld};

/// Helper: convert empty string to None for optional fields
fn empty_to_none(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

#[derive(Debug, Clone)]
pub struct WorldWithMap {
    pub world_id: i64,
    pub world_card: WorldCard,
    pub map: MapDef,
}

impl Storage {
    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, EngineError> {
        self.with_backend_mut(Operation::ListWorlds, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT key, name, description, global_rules, starting_room_id, scenarios, default_scenario_id, default_room_image, player_key FROM worlds",
                )?;
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, String>(8)?,
                    ))
                })?;
                rows.map(|r| {
                    let (key, name, description, global_rules_json, starting_room_id, scenarios_json, default_scenario_id, default_room_image, player_key) = r?;
                    Ok(WorldCard {
                        key,
                        name,
                        description,
                        global_rules: serde_json::from_str(&global_rules_json)
                            .map_err(|e| EngineError::Parse(format!("Failed to deserialize global_rules: {e}")))?,
                        starting_room_id,
                        scenarios: serde_json::from_str(&scenarios_json)
                            .map_err(|e| EngineError::Parse(format!("Failed to deserialize scenarios: {e}")))?,
                        default_scenario_id: default_scenario_id.filter(|s| !s.is_empty()),
                        default_room_image: default_room_image.filter(|s| !s.is_empty()),
                        player_key,
                    })
                }).collect()
            }
            Backend::InMemory(data) => Ok(data.worlds.iter().map(|w| w.world_card.clone()).collect()),
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, EngineError> {
        self.with_backend_mut(Operation::GetWorld, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT w.id, w.key, w.name, w.description, w.global_rules, w.starting_room_id, w.scenarios, w.default_scenario_id, w.default_room_image, w.player_key, m.map_data
                     FROM worlds w
                     LEFT JOIN maps m ON w.id = m.world_id
                     WHERE w.key = ?",
                )?;
                stmt.query_row([key], |row| {
                    let world_id = row.get::<_, i64>(0)?;
                    let key = row.get::<_, String>(1)?;
                    let name = row.get::<_, String>(2)?;
                    let description = row.get::<_, String>(3)?;
                    let global_rules_json = row.get::<_, String>(4)?;
                    let starting_room_id = row.get::<_, String>(5)?;
                    let scenarios_json = row.get::<_, String>(6)?;
                    let default_scenario_id = row.get::<_, Option<String>>(7)?;
                    let default_room_image = row.get::<_, Option<String>>(8)?;
                    let player_key = row.get::<_, String>(9)?;
                    let map_data_json = row.get::<_, String>(10)?;

                    let global_rules: Vec<String> = serde_json::from_str(&global_rules_json)
                        .map_err(|_e| rusqlite::Error::InvalidColumnType(4, "global_rules".into(), rusqlite::types::Type::Text))?;
                    let scenarios: Vec<_> = serde_json::from_str(&scenarios_json)
                        .map_err(|_e| rusqlite::Error::InvalidColumnType(6, "scenarios".into(), rusqlite::types::Type::Text))?;
                    let map: MapDef = serde_json::from_str(&map_data_json)
                        .map_err(|_e| rusqlite::Error::InvalidColumnType(10, "map_data".into(), rusqlite::types::Type::Text))?;

                    Ok(WorldWithMap {
                        world_id,
                        world_card: WorldCard {
                            key,
                            name,
                            description,
                            global_rules,
                            starting_room_id,
                            scenarios,
                            default_scenario_id: default_scenario_id.filter(|s| !s.is_empty()),
                            default_room_image: default_room_image.filter(|s| !s.is_empty()),
                            player_key,
                        },
                        map,
                    })
                })
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    rusqlite::Error::InvalidColumnType(_, field, _) => {
                        Err(EngineError::Parse(format!("Failed to deserialize {field}: JSON parse error")))
                    }
                    e => Err(EngineError::Database(e))
                })
            }
            Backend::InMemory(data) => Ok(
                data.worlds
                    .iter()
                    .find(|w| w.world_card.key == key)
                    .map(|w| WorldWithMap {
                        world_id: w.world_id,
                        world_card: w.world_card.clone(),
                        map: w.map.clone(),
                    })
            ),
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn seed_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::SeedWorld, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();
                let scenarios_json = serde_json::to_string(&world_card.scenarios)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize scenarios: {e}")))?;
                let global_rules_json = serde_json::to_string(&world_card.global_rules)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize global_rules: {e}")))?;

                conn.execute(
                    "INSERT OR IGNORE INTO worlds (key, name, description, global_rules, starting_room_id, scenarios, default_scenario_id, default_room_image, player_key, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        world_card.key,
                        world_card.name,
                        world_card.description,
                        global_rules_json,
                        world_card.starting_room_id,
                        scenarios_json,
                        empty_to_none(world_card.default_scenario_id.as_deref().unwrap_or("")),
                        empty_to_none(world_card.default_room_image.as_deref().unwrap_or("")),
                        &world_card.player_key,
                        now,
                        now,
                    ],
                )?;

                let world_id: i64 = conn.query_row(
                    "SELECT id FROM worlds WHERE key = ?",
                    [&world_card.key],
                    |row| row.get(0),
                )?;

                let map_data_json = serde_json::to_string(map)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize map: {e}")))?;

                conn.execute(
                    "INSERT OR IGNORE INTO maps (world_id, map_data, created_at, updated_at)
                     VALUES (?, ?, ?, ?)",
                    rusqlite::params![world_id, map_data_json, now, now],
                )?;

                Ok(())
            }
            Backend::InMemory(data) => {
                let world_id = (data.worlds.len() + 1) as i64;
                if !data.worlds.iter().any(|w| w.world_card.key == world_card.key) {
                    data.worlds.push(InMemoryWorld {
                        world_id,
                        world_card: world_card.clone(),
                        map: map.clone(),
                    });
                }
                Ok(())
            }
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn get_world_id(&self, key: &str) -> Result<Option<i64>, EngineError> {
        self.with_backend_mut(Operation::GetWorldId, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let result = conn.query_row("SELECT id FROM worlds WHERE key = ?", [key], |row| {
                    row.get(0)
                });
                match result {
                    Ok(id) => Ok(Some(id)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Database(e)),
                }
            }
            Backend::InMemory(data) => Ok(data
                .worlds
                .iter()
                .find(|w| w.world_card.key == key)
                .map(|w| w.world_id)),
            Backend::Test { .. } => unimplemented!(),
        })
    }
}
