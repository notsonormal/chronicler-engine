//! [DOC: docs/diataxis/reference/storage.md]
//! World storage backend operations

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;

use crate::error::EngineError;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::world::WorldCard;
use crate::adapters::driven::storage::{Backend, Storage};
use crate::adapters::driven::storage::in_memory_data::InMemoryWorld;
use crate::adapters::driven::storage::models::map::DbMap;
use crate::adapters::driven::storage::models::world::DbWorld;

#[derive(Debug, Clone)]
pub struct WorldWithMap {
    pub world_id: i64,
    pub world_card: WorldCard,
    pub map: MapDef,
}

#[derive(Debug, Clone)]
pub struct WorldBundle {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub persona: Arc<PersonaCard>,
    pub npcs: HashMap<String, NpcCard>,
}

impl Storage {
    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, EngineError> {
        self.with_backend_mut("list_worlds", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, key, name, description, global_rules, scenarios, default_scenario_id, default_room_image, created_at, updated_at FROM worlds",
                )?;
                let rows = stmt.query_map([], DbWorld::from_row)?;
                rows.map(|r| {
                    let db = r?;
                    db.to_card()
                }).collect()
            }
            Backend::InMemory(data) => Ok(data.worlds.iter().map(|w| w.world_card.clone()).collect()),
        })
    }

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, EngineError> {
        self.with_backend_mut("get_world", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut world_stmt = conn.prepare(
                    "SELECT id, key, name, description, global_rules, scenarios, default_scenario_id, default_room_image, created_at, updated_at
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

                let world_card = db_world.to_card()?;
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
        })
    }

    pub fn require_world(&self, key: &str) -> Result<WorldWithMap, EngineError> {
        self.get_world(key)?
            .ok_or_else(|| EngineError::WorldNotFound(key.to_string()))
    }

    pub fn world_bundle_for(&self, game_id: u64) -> Result<WorldBundle, EngineError> {
        let game = self.require_game(game_id)?;
        let world_with_map = self.require_world(&game.world_key)?;
        let persona = self.require_persona(&game.persona_key)?;
        let npcs: HashMap<String, NpcCard> = self
            .list_characters(world_with_map.world_id)?
            .into_iter()
            .map(|n| (n.id.clone(), n))
            .collect();
        Ok(WorldBundle {
            world: Arc::new(world_with_map.world_card),
            map: Arc::new(world_with_map.map),
            persona: Arc::new(persona),
            npcs,
        })
    }

    pub fn seed_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
        self.with_backend_mut("seed_world", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();

                conn.execute(
                    "INSERT OR REPLACE INTO worlds (
                        key, name, description, global_rules,
                        scenarios, default_scenario_id, default_room_image,
                        created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                    rusqlite::params![
                        world_card.key,
                        world_card.name,
                        world_card.description,
                        serde_json::to_string(&world_card.global_rules)?,
                        serde_json::to_string(&world_card.scenarios)?,
                        world_card.default_scenario_id.clone().unwrap_or_default(),
                        world_card.default_room_image.clone().unwrap_or_default(),
                        &now,
                    ],
                )
                .map_err(EngineError::Database)?;

                let world_id = conn.last_insert_rowid();

                conn.execute(
                    "INSERT OR REPLACE INTO maps (world_id, map_data, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?3)",
                    rusqlite::params![world_id, serde_json::to_string(map)?, &now],
                )
                .map_err(EngineError::Database)?;

                Ok(world_id)
            }
            Backend::InMemory(data) => {
                let world_id = data
                    .worlds
                    .iter()
                    .find(|w| w.world_card.key == world_card.key)
                    .map(|w| w.world_id)
                    .unwrap_or_else(|| {
                        let new_id = data.worlds.last().map(|w| w.world_id).unwrap_or(0) + 1;
                        data.worlds.push(InMemoryWorld {
                            world_id: new_id,
                            world_card: world_card.clone(),
                            map: map.clone(),
                        });
                        new_id
                    });
                Ok(world_id)
            }
        })
    }

    pub fn create_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
        self.seed_world(world_card, map) // Reuse idempotent seeding
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: &WorldCard,
        map: &MapDef,
    ) -> Result<(), EngineError> {
        self.with_backend_mut("update_world", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE worlds SET key=?, name=?, description=?, global_rules=?, scenarios=?, default_scenario_id=?, default_room_image=?, updated_at=? WHERE id=?",
                    rusqlite::params![
                        world_card.key, world_card.name, world_card.description,
                        serde_json::to_string(&world_card.global_rules)?,
                        serde_json::to_string(&world_card.scenarios)?,
                        world_card.default_scenario_id.clone().unwrap_or_default(),
                        world_card.default_room_image.clone().unwrap_or_default(),
                        &now, &id
                    ],
                )?;
                conn.execute(
                    "UPDATE maps SET map_data=?, updated_at=? WHERE world_id=?",
                    rusqlite::params![serde_json::to_string(map)?, &now, &id],
                )?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(inmem_world) = data.worlds.iter_mut().find(|w| w.world_id == id) {
                    inmem_world.world_card = world_card.clone();
                    inmem_world.map = map.clone();
                }
                Ok(())
            }
        })
    }

    pub fn get_world_by_id(&self, id: i64) -> Result<Option<WorldWithMap>, EngineError> {
        self.with_backend_mut("get_world_by_id", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                // Separate statements avoid column-index conflicts in `DbWorld::from_row` vs `DbMap::from_row`.
                let mut world_stmt = conn.prepare(
                    "SELECT id, key, name, description, global_rules, scenarios, default_scenario_id, default_room_image, created_at, updated_at
                     FROM worlds WHERE id = ?",
                )?;
                let db_world = match world_stmt.query_row([id], DbWorld::from_row) {
                    Ok(w) => w,
                    Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                    Err(e) => return Err(EngineError::Database(e)),
                };

                let mut map_stmt = conn.prepare(
                    "SELECT id, world_id, map_data, created_at, updated_at FROM maps WHERE world_id = ?",
                )?;
                let db_map = map_stmt.query_row([db_world.id], DbMap::from_row)
                    .map_err(EngineError::Database)?;

                let world_card = db_world.to_card()?;
                let map: MapDef = serde_json::from_str(&db_map.map_data)
                    .map_err(|e| EngineError::Parse(format!("Failed to deserialize map: {e}")))?;

                Ok(Some(WorldWithMap { world_id: db_world.id, world_card, map }))
            }
            Backend::InMemory(data) => {
                Ok(data.worlds
                    .iter()
                    .find(|w| w.world_id == id)
                    .map(|w| WorldWithMap {
                        world_id: w.world_id,
                        world_card: w.world_card.clone(),
                        map: w.map.clone(),
                    }))
            }
        })
    }

    pub fn delete_world(&self, key: &str) -> Result<(), EngineError> {
        self.with_backend_mut("delete_world", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM games WHERE world_key = ?",
                    [key],
                    |row| row.get(0),
                )?;
                if count > 0 {
                    return Err(EngineError::WorldHasGames {
                        game_count: count as usize,
                    });
                }
                conn.execute("DELETE FROM worlds WHERE key = ?", [key])?;
                Ok(())
            }
            Backend::InMemory(data) => {
                let game_count = data.games.iter().filter(|g| g.world_key == key).count();
                if game_count > 0 {
                    return Err(EngineError::WorldHasGames { game_count });
                }
                data.worlds.retain(|w| w.world_card.key != key);
                Ok(())
            }
        })
    }
}
