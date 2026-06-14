# Multi-World Data Foundation

## Context

The Chronicler Engine currently selects a single world at CLI startup (`--world redmist_estate`) and bakes it into `AppState` as `Arc<WorldCard>`, `Arc<MapDef>`, `Arc<PlayerCard>`, `Arc<HashMap<String, NpcCard>>` singletons. All games in a server instance belong to that one world; `switch_game()` rejects cross-world switches. This plan refactors the data layer so that world context is loaded from the DB per request — enabling games from different worlds to coexist and be switched between. No UI changes; those follow in separate plans.

## Approach

### Step 1: Migration v12 — Add `world_key` column to `games` table

**Edit**: `src/storage/db.rs`

Add `if version < 12` block after the `version < 10` block:

```rust
if version < 12 {
    let exec = |sql: &str| {
        conn.execute(sql, [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
    };

    // Add world_key column for stable world references
    exec("ALTER TABLE games ADD COLUMN world_key TEXT NOT NULL DEFAULT ''")?;
    
    // Backfill: match world_name to worlds.key
    // For 'default' or unmatched: use 'redmist_estate'
    exec("UPDATE games SET world_key = COALESCE(
        (SELECT key FROM worlds WHERE worlds.name = games.world_name),
        'redmist_estate'
    ) WHERE world_key = ''")?;

    conn.pragma_update(None, "user_version", 12).map_err(|e| {
        crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
    })?;
}
```

No FOREIGN KEY constraint — `world_key` is a logical reference, same pattern as `world_name`.

### Step 2: Add `world_key` to Game domain model and DB model

**Edit**: `src/model/game.rs`
```rust
pub struct Game {
    pub id: u64,
    pub name: String,
    pub world_name: String,
    pub world_key: String,  // NEW
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Edit**: `src/storage/models/game.rs`
```rust
pub struct DbGame {
    pub id: i64,
    pub world_name: String,
    pub name: String,
    pub world_key: String,  // NEW
    pub created_at: String,
    pub updated_at: String,
}
```

Update `DbGame::from_row()` to read the new column (column index 3, after `name`).

**Edit**: `src/storage/backend/games.rs`
```rust
fn db_game_to_game(db: &DbGame) -> Result<Game, EngineError> {
    Ok(Game {
        id: db.id as u64,
        world_name: db.world_name.clone(),
        world_key: db.world_key.clone(),  // NEW
        name: db.name.clone(),
        created_at: parse_datetime(&db.created_at, "created_at")?,
        updated_at: parse_datetime(&db.updated_at, "updated_at")?,
    })
}
```

**Edit**: `src/storage/backend/games.rs` — `create_game()` signature:
```rust
pub fn create_game(&self, world_name: &str, world_key: &str, name: &str) -> Result<u64, EngineError> {
    self.with_backend_mut(Operation::CreateGame, |backend, _game_id| match backend {
        Backend::Sqlite { pool } => {
            let conn = pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                rusqlite::params![world_name, world_key, name, &now],
            )?;
            Ok(conn.last_insert_rowid() as u64)
        }
        // ... InMemory branch
    })
}
```

**Callsite updates**:
- `src/application/game_lifecycle.rs:26` — `ctx.storage.create_game(&world_name, &world_key, &name)`
- `src/bootstrap/run.rs` ~line 214 — Add `world_key` to INSERT statement and params

### Step 3: Add `create_world` and `update_world` storage methods

**Edit**: `src/storage/backend/worlds.rs`

```rust
pub fn create_world(&self, world_card: &WorldCard, map: &MapDef) -> Result<i64, EngineError> {
    self.seed_world(world_card, map)  // Reuse idempotent seeding
}

pub fn update_world(&self, id: i64, world_card: &WorldCard, map: &MapDef) -> Result<(), EngineError> {
    self.with_backend_mut(Operation::UpdateWorld, |backend, _game_id| match backend {
        Backend::Sqlite { pool } => {
            let conn = pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE worlds SET key=?, name=?, description=?, global_rules=?, starting_room_id=?, scenarios=?, default_scenario_id=?, default_room_image=?, player_key=?, updated_at=? WHERE id=?",
                rusqlite::params![
                    world_card.key, world_card.name, world_card.description,
                    serde_json::to_string(&world_card.global_rules)?,
                    world_card.starting_room_id,
                    serde_json::to_string(&world_card.scenarios)?,
                    world_card.default_scenario_id.clone().unwrap_or_default(),
                    world_card.default_room_image.clone().unwrap_or_default(),
                    world_card.player_key, &now, &id
                ],
            )?;
            conn.execute(
                "UPDATE maps SET map_data=?, updated_at=? WHERE world_id=?",
                rusqlite::params![serde_json::to_string(map)?, &now, &id],
            )?;
            Ok(())
        }
        _ => unreachable!(),
    })
}

pub fn get_world_by_id(&self, id: i64) -> Result<Option<WorldWithMap>, EngineError> {
    self.with_backend_mut(Operation::GetWorldById, |backend, _game_id| match backend {
        Backend::Sqlite { pool } => {
            let conn = pool.conn();
            let mut stmt = conn.prepare(
                "SELECT w.id, w.key, w.name, w.description, w.global_rules, w.starting_room_id, w.scenarios, w.default_scenario_id, w.default_room_image, w.player_key, w.created_at, w.updated_at, m.id, m.map_data, m.created_at, m.updated_at FROM worlds w JOIN maps m ON m.world_id = w.id WHERE w.id = ?"
            )?;
            let rows = stmt.query_map([id], |row| {
                Ok((DbWorld::from_row(row)?, DbMap::from_row(row)?))
            })?;
            // ... same mapping logic as get_world()
        }
        _ => unreachable!(),
    })
}
```

**Edit**: `src/storage/backend/core.rs` — Add `Operation::UpdateWorld` and `Operation::GetWorldById` variants to the enum.

### Step 4: Refactor `AppState` — Load world data from DB per request

**Edit**: `src/server/app_state.rs`

Remove from `AppState`:
```rust
pub world: Arc<WorldCard>,
pub map: Arc<MapDef>,
pub player: Arc<PlayerCard>,
pub npcs: Arc<HashMap<String, NpcCard>>,
```

Remove same fields from `ServerResources`.

Change `as_game_service_context()`:
```rust
pub fn as_game_service_context(&self) -> Result<GameServiceContext, EngineError> {
    let game_id = self.storage.current_game_id();
    let game = self.storage.get_game(game_id)?.ok_or_else(|| EngineError::Config("No active game"))?;
    let world_with_map = self.storage.get_world(&game.world_key)?
        .ok_or_else(|| EngineError::Config(format!("World not found: {}", game.world_key)))?;
    let player = self.storage.get_persona(&world_with_map.world_card.player_key)?
        .ok_or_else(|| EngineError::Config(format!("Persona not found: {}", world_with_map.world_card.player_key)))?;
    let npcs = self.storage.list_characters(world_with_map.world_id)?;
    let npcs_map: HashMap<String, NpcCard> = npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect();
    
    Ok(GameServiceContext {
        storage: Arc::clone(&self.storage),
        world: Arc::new(world_with_map.world_card),
        map: Arc::new(world_with_map.map),
        player: Arc::new(player),
        npcs: Arc::new(npcs_map),
        cancel_token: self.current_cancel_token(),
        is_generating: Arc::clone(&self.is_generating),
        settings: Arc::clone(&self.settings),
        preset_storage: Arc::clone(&self.preset_storage),
    })
}

pub fn as_game_service_context_or_default(&self) -> GameServiceContext {
    self.as_game_service_context().unwrap_or_else(|_| GameServiceContext {
        storage: Arc::clone(&self.storage),
        world: Arc::new(WorldCard::default()),
        map: Arc::new(MapDef { overworld: Overworld { id: String::new(), name: String::new(), regions: vec![] } }),
        player: Arc::new(PlayerCard::default()),
        npcs: Arc::new(HashMap::new()),
        cancel_token: self.current_cancel_token(),
        is_generating: Arc::clone(&self.is_generating),
        settings: Arc::clone(&self.settings),
        preset_storage: Arc::clone(&self.preset_storage),
    })
}
```

**Callsite pattern**:
- Handlers with error branches: `state.as_game_service_context()?`
- Non-critical fragments (renderers): `state.as_game_service_context_or_default()`

### Step 5: Enable cross-world game switching

**Edit**: `src/application/game_lifecycle.rs`

`switch_game()` — remove lines 72-75:
```rust
// REMOVE THIS BLOCK:
// if game.world_name != ctx.world.name {
//     return Err(ApplicationError::validation("Game belongs to a different world"));
// }
```

`create_game()` — new signature:
```rust
pub fn create_game(&self, ctx: GameServiceContext, world_key: &str) -> Result<u64, ApplicationError> {
    if ctx.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(ApplicationError::ConcurrentGeneration);
    }

    let world_with_map = ctx.storage.get_world(world_key)?
        .ok_or_else(|| ApplicationError::validation("World not found"))?;
    let world_name = world_with_map.world_card.name.clone();
    let games = ctx.storage.list_games()?;
    let existing_names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
    let name = generate_game_name(&world_name, &existing_names);

    let new_id = ctx.storage.create_game(&world_name, world_key, &name)?;
    // ... rest unchanged
}
```

**Edit**: `src/application/application_service.rs` — pass `world_key` through.

### Step 6: Bootstrap startup fallback

**Edit**: `src/bootstrap/run.rs`

```rust
// After seed_game_data():
let world_with_map = match lookup_storage.get_world(&args.world)? {
    Some(w) => w,
    None => {
        let all_worlds = lookup_storage.list_worlds()?;
        if all_worlds.is_empty() {
            return Err(EngineError::Config("No worlds available in database".to_string()));
        }
        tracing::warn!("World '{}' not found, falling back to '{}'", args.world, all_worlds[0].key);
        lookup_storage.get_world(&all_worlds[0].key)?.unwrap()
    }
};
```

**Edit**: `src/server/server_impl.rs` — Remove `world`, `map`, `player`, `npcs` from `AppState::new()`.

### Step 7: Test updates

Search for `as_game_service_context`, `create_game`, `AppState {` in test files and update:
- Tests calling `as_game_service_context()` must handle `Result` or use `unwrap()`
- Tests constructing `AppState` directly must remove world/map/player/npcs fields
- Tests calling `create_game()` must add `world_key` parameter

## Critical Files & Anchors

- `src/server/app_state.rs:45-73` — `AppState` struct and `as_game_service_context()`.
- `src/storage/db.rs:154-248` — Migration v10/v11 blocks. Insert migration v12 here.
- `src/application/game_lifecycle.rs:16-63` — `create_game()`, `switch_game()`.
- `src/storage/backend/games.rs:49-75` — `create_game()`.
- `src/bootstrap/run.rs:158-221` — Startup world loading.

## Verification

1. **Migration**: `cargo run -- --world redmist_estate` → DB upgraded, `world_key` backfilled.
2. **Cross-world switch**: Create game under `test` world. Switch to it from `redmist_estate` game. Verify correct map/NPCs load.
3. **Fallback**: `cargo run -- --world nonexistent` → server starts with first available world.
4. **Build**: `python build.py` passes.
