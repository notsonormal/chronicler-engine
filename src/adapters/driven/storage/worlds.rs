//! [DOC: docs/diataxis/reference/storage.md]
//! World storage backend operations

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;

use crate::error::{DatabaseContext, EngineError};
use crate::domain::model::agent::WorldView;
use crate::domain::model::character::{CharacterCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::utils::game_rules::GameWorld;
use crate::domain::model::world::WorldCard;
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
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
    pub characters: HashMap<String, CharacterCard>,
}

impl WorldBundle {
    pub fn game_world(&self) -> GameWorld<'_> {
        GameWorld {
            map: &self.map,
            characters: &self.characters,
            persona_name: &self.persona.sheet.name,
        }
    }

    /// What a hook reads of the World, with the Characters present in `state`.
    pub fn view<'a>(&'a self, state: &'a SwipeState) -> WorldView<'a> {
        WorldView {
            room: state.room(&self.map),
            map: &self.map,
            persona: &self.persona,
            characters: &self.characters,
            present: &state.scene.characters_present,
        }
    }
}

impl Storage {
    /// The `worlds` read query. `where_clause` is appended verbatim (empty or
    /// `WHERE ...`); the column list derives from `WORLD_COLUMNS`, so the read
    /// list cannot drift from the write list. `DbWorld::from_row` reads every
    /// column by name.
    fn world_select_sql(where_clause: &str) -> String {
        format!(
            "SELECT id, {}, created_at, updated_at FROM worlds {where_clause}",
            WORLD_COLUMNS.join(", ")
        )
    }

    pub fn list_worlds(&self) -> Result<Vec<WorldCard>, EngineError> {
        self.with_backend_mut(StorageCall::ListWorlds, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(&Self::world_select_sql(""))
                    .db_context("Failed to prepare the worlds query")?;
                let rows = stmt
                    .query_map([], DbWorld::from_row)
                    .db_context("Failed to query the worlds")?;
                rows.map(|r| r.db_context("Failed to read a world row")?.to_card())
                    .collect()
            }
        })
    }

    pub fn get_world(&self, key: &str) -> Result<Option<WorldWithMap>, EngineError> {
        self.with_backend_mut(StorageCall::GetWorld, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut world_stmt = conn
                    .prepare(&Self::world_select_sql("WHERE key = ?"))
                    .db_context("Failed to prepare the world query")?;
                let db_world = match world_stmt.query_row([key], DbWorld::from_row) {
                    Ok(w) => w,
                    Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                    Err(e) => return Err(e).db_context("Failed to read the world"),
                };

                let mut map_stmt = conn
                    .prepare(
                        "SELECT id, world_id, map_data, created_at, updated_at
                         FROM maps
                         WHERE world_id = ?",
                    )
                    .db_context("Failed to prepare the map query")?;
                let db_map = map_stmt
                    .query_row([db_world.id], DbMap::from_row)
                    .db_context("Failed to read the map")?;

                let world_card = db_world.to_card()?;
                let map: MapDef = serde_json::from_str(&db_map.map_data)
                    .map_err(|e| EngineError::Parse(format!("Failed to deserialize map: {e}")))?;

                Ok(Some(WorldWithMap {
                    world_id: db_world.id,
                    world_card,
                    map,
                }))
            }
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
        let characters: HashMap<String, CharacterCard> = self
            .list_characters(world_with_map.world_id)?
            .into_iter()
            .map(|n| (n.id.clone(), n))
            .collect();
        Ok(WorldBundle {
            world: Arc::new(world_with_map.world_card),
            map: Arc::new(world_with_map.map),
            persona: Arc::new(persona),
            characters,
        })
    }

    /// Upsert a world by key. Used by bootstrap seeding of `data/worlds/*`,
    /// where replacing an existing world's data is intended. The world row is
    /// updated in place, so its id and dependent characters survive.
    pub fn seed_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
        self.with_backend_mut(StorageCall::SeedWorld, |backend| {
            Self::write_world(backend, world_card, map, WorldWriteMode::Upsert)
        })
    }

    /// Create a world, refusing a key that already exists. The user-facing
    /// create path must never replace an authored world; bootstrap seeding
    /// uses `seed_world` for its replace-on-key upsert instead.
    pub fn create_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
        self.with_backend_mut(StorageCall::CreateWorld, |backend| {
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
                let tx = conn
                    .unchecked_transaction()
                    .db_context("Failed to begin the world write")?;
                let world_id = Self::write_world_row(&tx, world_card, &now, mode)?;
                Self::write_map_row(&tx, world_id, map, &now)?;
                tx.commit().db_context("Failed to commit the world write")?;
                Ok(world_id)
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
            _ => EngineError::Database {
                context: "Failed to write the world row",
                source: e,
            },
        })?;

        match mode {
            WorldWriteMode::Insert => Ok(conn.last_insert_rowid()),
            WorldWriteMode::Upsert => conn
                .query_row(
                    "SELECT id FROM worlds WHERE key = ?",
                    [&world_card.key],
                    |row| row.get(0),
                )
                .db_context("Failed to read the world id"),
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
        let updated = conn
            .execute(
                "UPDATE maps SET map_data = ?1, updated_at = ?2 WHERE world_id = ?3",
                rusqlite::params![map_data, now, world_id],
            )
            .db_context("Failed to update the map")?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO maps (world_id, map_data, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?3)",
                rusqlite::params![world_id, map_data, now],
            )
            .db_context("Failed to insert the map")?;
        }
        Ok(())
    }

    pub fn update_world(
        &self,
        id: i64,
        world_card: &WorldCard,
        map: &MapDef,
    ) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::UpdateWorld, |backend| match backend {
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
                )
                .db_context("Failed to update the world")?;
                Self::write_map_row(&conn, id, map, &now)?;
                Ok(())
            }
        })
    }

    pub fn get_world_by_id(&self, id: i64) -> Result<Option<WorldWithMap>, EngineError> {
        self.with_backend_mut(StorageCall::GetWorldById, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                // World and map are read with separate statements: `DbWorld`
                // reads by column name, `DbMap` positionally within its own
                // result set.
                let mut world_stmt = conn
                    .prepare(&Self::world_select_sql("WHERE id = ?"))
                    .db_context("Failed to prepare the world query by id")?;
                let db_world = match world_stmt.query_row([id], DbWorld::from_row) {
                    Ok(w) => w,
                    Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                    Err(e) => return Err(e).db_context("Failed to read the world by id"),
                };

                let mut map_stmt = conn
                    .prepare(
                        "SELECT id, world_id, map_data, created_at, updated_at FROM maps WHERE world_id = ?",
                    )
                    .db_context("Failed to prepare the map query by world id")?;
                let db_map = map_stmt
                    .query_row([db_world.id], DbMap::from_row)
                    .db_context("Failed to read the map by world id")?;

                let world_card = db_world.to_card()?;
                let map: MapDef = serde_json::from_str(&db_map.map_data)
                    .map_err(|e| EngineError::Parse(format!("Failed to deserialize map: {e}")))?;

                Ok(Some(WorldWithMap { world_id: db_world.id, world_card, map }))
            }
        })
    }

    pub fn delete_world(&self, key: &str) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::DeleteWorld, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM games WHERE world_key = ?",
                        [key],
                        |row| row.get(0),
                    )
                    .db_context("Failed to count the games of the world")?;
                if count > 0 {
                    return Err(EngineError::WorldHasGames {
                        game_count: count as usize,
                    });
                }
                conn.execute("DELETE FROM worlds WHERE key = ?", [key])
                    .db_context("Failed to delete the world")?;
                Ok(())
            }
        })
    }
}

/// The `worlds` columns every write binds, in `params!` order. The insert
/// column list, the upsert set, `update_world`'s set and the read query all
/// derive from it, so a new column is listed here once.
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
