//! [DOC: docs/system/storage.md]
//! World storage backend operations

use chrono::Utc;

use crate::error::EngineError;
use crate::model::map::MapDef;
use crate::model::world::WorldCard;
use crate::storage::backend::{empty_to_none, Backend, Operation, Storage, InMemoryWorld};
use crate::storage::models::world::{DbWorld, DbMap};

pub(crate) fn world_card_from_db(db: &DbWorld) -> Result<WorldCard, EngineError> {
    Ok(WorldCard {
        key: db.key.clone(),
        name: db.name.clone(),
        description: db.description.clone(),
        global_rules: serde_json::from_str(&db.global_rules)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize global_rules: {e}")))?,
        starting_room_id: db.starting_room_id.clone(),
        scenarios: serde_json::from_str(&db.scenarios)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize scenarios: {e}")))?,
        default_scenario_id: db.default_scenario_id.clone().filter(|s| !s.is_empty()),
        default_room_image: db.default_room_image.clone().filter(|s| !s.is_empty()),
        player_key: db.player_key.clone(),
    })
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
                    "SELECT id, key, name, description, global_rules, starting_room_id, scenarios, default_scenario_id, default_room_image, player_key, created_at, updated_at FROM worlds",
                )?;
                let rows = stmt.query_map([], DbWorld::from_row)?;
                rows.map(|r| {
                    let db = r?;
                    world_card_from_db(&db)
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
                let mut world_stmt = conn.prepare(
                    "SELECT id, key, name, description, global_rules, starting_room_id, scenarios, default_scenario_id, default_room_image, player_key, created_at, updated_at
                     FROM worlds
                     WHERE key = ?",
                )?;
                let db_world = match world_stmt.query_row([key], DbWorld::from_row) {
                    Ok(w) => w,
                    Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                    Err(e) => return Err(EngineError::Database(e)),
                };

                let mut map_stmt = conn.prepare(
                    "SELECT id, world_id, map_data, created_at, updated_at
                     FROM maps
                     WHERE world_id = ?",
                )?;
                let db_map = map_stmt.query_row([db_world.id], DbMap::from_row)
                    .map_err(EngineError::Database)?;

                let world_card = world_card_from_db(&db_world)?;
                let map: MapDef = serde_json::from_str(&db_map.map_data)
                    .map_err(|e| EngineError::Parse(format!("Failed to deserialize map: {e}")))?;

                Ok(Some(WorldWithMap {
                    world_id: db_world.id,
                    world_card,
                    map,
                }))
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

    pub fn seed_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
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

                Ok(world_id)
            }
            Backend::InMemory(data) => {
                if let Some(existing) = data.worlds.iter().find(|w| w.world_card.key == world_card.key) {
                    Ok(existing.world_id)
                } else {
                    let world_id = (data.worlds.len() + 1) as i64;
                    data.worlds.push(InMemoryWorld {
                        world_id,
                        world_card: world_card.clone(),
                        map: map.clone(),
                    });
                    Ok(world_id)
                }
            }
            Backend::Test { .. } => unimplemented!(),
        })
    }
}
