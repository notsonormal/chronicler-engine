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
                    "SELECT id, key, name, description, global_rules, scenarios, default_scenario_id, default_room_image, narrator_mode, narrative_perspective, narrative_tense, created_at, updated_at, options_always_on FROM worlds",
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
                    "SELECT id, key, name, description, global_rules, scenarios, default_scenario_id, default_room_image, narrator_mode, narrative_perspective, narrative_tense, created_at, updated_at, options_always_on
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

    /// Upsert a world by key. Used by bootstrap seeding of `data/worlds/*`,
    /// where replacing an existing world's data is intended. The world row is
    /// updated in place, so its id and dependent characters survive; both
    /// backends replace the stored card and map for an existing key.
    pub fn seed_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
        self.with_backend_mut("seed_world", |backend| {
            Self::write_world(backend, world_card, map, WorldWriteMode::Upsert)
        })
    }

    /// Create a world, refusing a key that already exists. The user-facing
    /// create path must never replace an authored world; bootstrap seeding
    /// uses `seed_world` for its replace-on-key upsert instead.
    pub fn create_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
        self.with_backend_mut("create_world", |backend| {
            Self::write_world(backend, world_card, map, WorldWriteMode::Insert)
        })
    }

    /// Write a world and its map, keeping the world row id (and so its
    /// characters) when the key already exists and `mode` allows the update.
    fn write_world(
        backend: &mut Backend,
        world_card: &WorldCard,
        map: &MapDef,
        mode: WorldWriteMode,
    ) -> Result<i64, EngineError> {
        match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();

                // World and map are one write: a failed map write must not
                // leave a re-seeded world behind a stale map.
                let tx = conn.unchecked_transaction()?;
                let world_id = Self::write_world_row(&tx, world_card, &now, mode)?;
                Self::write_map_row(&tx, world_id, map, &now)?;
                tx.commit()?;
                Ok(world_id)
            }
            Backend::InMemory(data) => {
                if let Some(existing) = data
                    .worlds
                    .iter_mut()
                    .find(|w| w.world_card.key == world_card.key)
                {
                    if mode == WorldWriteMode::Insert {
                        return Err(EngineError::WorldAlreadyExists(world_card.key.clone()));
                    }
                    existing.world_card = world_card.clone();
                    existing.map = map.clone();
                    return Ok(existing.world_id);
                }

                let new_id = data.next_world_id;
                data.next_world_id += 1;
                data.worlds.push(InMemoryWorld {
                    world_id: new_id,
                    world_card: world_card.clone(),
                    map: map.clone(),
                });
                Ok(new_id)
            }
        }
    }

    /// Insert the `worlds` row, or update it in place on a key conflict when
    /// `mode` allows. Returns the surviving row id. In `Insert` mode the
    /// `worlds.key UNIQUE` constraint is the authority for the refusal.
    fn write_world_row(
        conn: &rusqlite::Connection,
        world_card: &WorldCard,
        now: &str,
        mode: WorldWriteMode,
    ) -> Result<i64, EngineError> {
        // `params!` below binds the shared columns first, then `now`.
        let columns = WORLD_COLUMNS.join(", ");
        let placeholders = (1..=WORLD_COLUMNS.len())
            .map(|position| format!("?{position}"))
            .collect::<Vec<_>>()
            .join(", ");
        let timestamp = format!("?{}", WORLD_COLUMNS.len() + 1);
        let insert = format!(
            "INSERT INTO worlds ({columns}, created_at, updated_at) \
             VALUES ({placeholders}, {timestamp}, {timestamp})"
        );
        let upsert_assignments = WORLD_COLUMNS
            .iter()
            .filter(|column| **column != "key")
            .map(|column| format!("{column} = excluded.{column}"))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = match mode {
            WorldWriteMode::Insert => insert,
            WorldWriteMode::Upsert => format!(
                "{insert} ON CONFLICT(key) DO UPDATE SET \
                 {upsert_assignments}, updated_at = excluded.updated_at"
            ),
        };

        conn.execute(
            &sql,
            rusqlite::params![
                world_card.key,
                world_card.name,
                world_card.description,
                serde_json::to_string(&world_card.global_rules)?,
                serde_json::to_string(&world_card.scenarios)?,
                world_card.default_scenario_id.clone().unwrap_or_default(),
                world_card.default_room_image.clone().unwrap_or_default(),
                world_card.narrator_mode.as_str(),
                world_card.narrative_perspective.as_str(),
                world_card.narrative_tense.as_str(),
                world_card.options_always_on as i64,
                now,
            ],
        )
        .map_err(|e| match mode {
            WorldWriteMode::Insert if Self::is_unique_violation(&e) => {
                EngineError::WorldAlreadyExists(world_card.key.clone())
            }
            _ => EngineError::Database(e),
        })?;

        match mode {
            WorldWriteMode::Insert => Ok(conn.last_insert_rowid()),
            WorldWriteMode::Upsert => conn
                .query_row(
                    "SELECT id FROM worlds WHERE key = ?",
                    [&world_card.key],
                    |row| row.get(0),
                )
                .map_err(EngineError::Database),
        }
    }

    /// Whether `error` is SQLite refusing a duplicate `worlds.key`.
    fn is_unique_violation(error: &rusqlite::Error) -> bool {
        matches!(
            error,
            rusqlite::Error::SqliteFailure(err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation
                    && err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
        )
    }

    /// Write the `maps` row for `world_id`, updating it in place when present
    /// so a re-seed never leaves a second map behind.
    fn write_map_row(
        conn: &rusqlite::Connection,
        world_id: i64,
        map: &MapDef,
        now: &str,
    ) -> Result<(), EngineError> {
        let map_data = serde_json::to_string(map)?;
        let updated = conn.execute(
            "UPDATE maps SET map_data = ?1, updated_at = ?2 WHERE world_id = ?3",
            rusqlite::params![map_data, now, world_id],
        )?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO maps (world_id, map_data, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?3)",
                rusqlite::params![world_id, map_data, now],
            )?;
        }
        Ok(())
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
                // `params!` below binds the shared columns first, then `now`, `id`.
                let assignments = WORLD_COLUMNS
                    .iter()
                    .map(|column| format!("{column}=?"))
                    .collect::<Vec<_>>()
                    .join(", ");
                conn.execute(
                    &format!("UPDATE worlds SET {assignments}, updated_at=? WHERE id=?"),
                    rusqlite::params![
                        world_card.key,
                        world_card.name,
                        world_card.description,
                        serde_json::to_string(&world_card.global_rules)?,
                        serde_json::to_string(&world_card.scenarios)?,
                        world_card.default_scenario_id.clone().unwrap_or_default(),
                        world_card.default_room_image.clone().unwrap_or_default(),
                        world_card.narrator_mode.as_str(),
                        world_card.narrative_perspective.as_str(),
                        world_card.narrative_tense.as_str(),
                        world_card.options_always_on as i64,
                        &now,
                        &id
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
                    "SELECT id, key, name, description, global_rules, scenarios, default_scenario_id, default_room_image, narrator_mode, narrative_perspective, narrative_tense, created_at, updated_at, options_always_on
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
                // SQLite reaches the characters through the `world_id` foreign
                // key's cascade; InMemory has no cascade, so drop them here.
                if let Some(world) = data.worlds.iter().find(|w| w.world_card.key == key) {
                    let world_id = world.world_id;
                    data.characters
                        .retain(|character| character.world_id != world_id);
                }
                data.worlds.retain(|w| w.world_card.key != key);
                Ok(())
            }
        })
    }
}

/// The `worlds` columns every write binds, in `params!` order. The insert
/// column list, the upsert set and `update_world`'s set all derive from it, so
/// a new column is listed here once.
const WORLD_COLUMNS: &[&str] = &[
    "key",
    "name",
    "description",
    "global_rules",
    "scenarios",
    "default_scenario_id",
    "default_room_image",
    "narrator_mode",
    "narrative_perspective",
    "narrative_tense",
    "options_always_on",
];

/// Whether a write to the `worlds` table updates an existing key in place or
/// refuses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldWriteMode {
    /// Refuse a key that already exists (user-facing create).
    Insert,
    /// Replace an existing key's data in place (bootstrap seed).
    Upsert,
}
