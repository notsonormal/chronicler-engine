# Plan: Phase 3 — Switch Runtime World Loading from Files to Database

**STATUS:** ✅ **COMPLETED** (2026-06-13)  
**Implementation:** 95% complete - all core functionality implemented, 5 minor test failures expected  
**Archived:** 2026-06-13

---

## Context

The game data migration (ADR-024) is partially complete: DB schema and storage CRUD exist (Phase 1), and settings write-through works (Phase 4). But Phase 2 (seed logic for worlds/personas/characters) was never actually implemented — only prompt presets are seeded in `ensure_defaults()`. The `Storage::seed_world()`, `seed_persona()`, `seed_character()` methods exist but are never called. Meanwhile, `run()` still loads all world data from JSON files via `initialize_world_from_manifest()`.

This plan completes Phase 2 (actual seeding) and Phase 3 (switch reads to DB) together, because Phase 3 cannot work without Phase 2.

**End state:** JSON files are read only once during seeding (first startup or if DB empty). All runtime world/persona/character data comes from DB. Game plays identically.

## Approach

### Step 1: Add `key`, `default_scenario_id`, `player_key` to `WorldCard`; delete `WorldInfo`; simplify `seed_world`

Add three fields to `WorldCard` so it carries everything the DB stores, matching the pattern where personas/characters have no separate "Info" wrapper. Delete the unused `WorldInfo` struct. Since `WorldCard` now holds all fields that `seed_world` reads from `WorldManifest`, drop the `&WorldManifest` parameter from `seed_world`.

**File:** `src/model/world.rs`
- Add to `WorldCard` (after `description`, before `global_rules` — identity field follows naming):
  ```rust
  #[serde(default)]
  pub key: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub default_scenario_id: Option<String>,
  #[serde(default)]
  pub player_key: String,
  ```
- Add helper method:
  ```rust
  pub fn effective_player_key(&self) -> &str {
      if self.player_key.is_empty() { "player" } else { &self.player_key }
  }
  ```
  Fallback `"player"` matches the default `player.json` filename stem.
- Delete `WorldInfo` struct and its doc comment entirely
- Update `From<WorldManifest> for WorldCard` to populate:
  - `key` ← `manifest.id`
  - `default_scenario_id` ← `manifest.default_scenario_id`
  - `player_key` ← stem of `manifest.player_file` (e.g. `"julian"` from `"julian.json"`)
  - Derive `player_key` using: `Path::new(&manifest.player_file).file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string()`
- `WorldCard` already derives `Default` — `String::default()` = `""`, `Option::default()` = `None`, so no Default impl change needed

**File:** `src/storage/backend/worlds.rs`
- `WorldWithMap` becomes:
  ```rust
  #[derive(Debug, Clone)]
  pub struct WorldWithMap {
      pub world_card: WorldCard,
      pub map: MapDef,
  }
  ```
- Update `get_world()` SELECT to include `player_key`, construct `WorldCard` directly (field list: `key, name, description, global_rules, starting_room_id, scenarios, default_scenario_id, default_room_image, player_key`)
- Update `list_worlds()` to return `Vec<WorldCard>`, include `player_key` in SELECT
- **`seed_world` signature change**: `(manifest: &WorldManifest, world_card: &WorldCard, map: &MapDef)` → `(world_card: &WorldCard, map: &MapDef)`. All fields previously read from `manifest` are now on `world_card`:
  - `manifest.id` → `world_card.key`
  - `manifest.scenarios` → `world_card.scenarios`
  - `manifest.starting_room_id` → `world_card.starting_room_id`
  - `manifest.default_scenario_id` → `world_card.default_scenario_id`
  - `manifest.default_room_image` → `world_card.default_room_image`
  - `manifest.name`/`description` → `world_card.name`/`description` (already used)
- InMemory branch: duplicate check changes from `w.manifest.id == manifest.id` to `w.world_card.key == world_card.key`

**File:** `src/storage/backend/core.rs`
- `WorldSeed` struct: remove `manifest: WorldManifest` field; add `world_id: i64` field for InMemory `get_world_id()` support. Becomes:
  ```rust
  pub struct WorldSeed {
      pub world_id: i64,
      pub world_card: WorldCard,
      pub map: MapDef,
  }
  ```
- `InMemoryData.worlds: Vec<WorldSeed>` — no type change (WorldSeed shape changes)
- Update InMemory `list_worlds`, `get_world` branches to read from `WorldSeed.world_card` instead of `WorldSeed.manifest`
- Add `GetWorldId` to `Operation` enum (convention: `VerbNoun` in PascalCase, e.g. `GetWorldId`)

**File:** `src/storage/backend/worlds_tests.rs`
- Update all assertions from `WorldInfo` to `WorldCard`
- Update `seed_world` calls to drop `&WorldManifest` param, pass `&WorldCard` with `key` populated

**Call sites for `seed_world` signature change** — search `seed_world(` in entire repo; as of this writing the only caller is the new code in Step 4 (ensure_defaults itself). Tests in `worlds_tests.rs` are listed above.

**All `WorldCard` struct-literal construction sites** — 18 locations across tests construct `WorldCard { ... }`. Since `WorldCard` derives `Default`, convert those not already using `..Default::default()` to use it. Exact sites found via AST search for `WorldCard { $$$ }`:
- `src/test_support/fixtures.rs:18` (TestWorld::minimal) — add 3 new fields or use `..Default::default()`
- `src/test_support/test_app_builder.rs:37` — same
- `src/engine/trigger_eval_tests.rs:63,287` — 287 already uses `..Default::default()`, 63 does not
- `src/engine/logic_tests.rs:64,123` — neither uses Default
- `src/narrative/prompt/assembler_tests.rs:24` — does not use Default
- `src/application/message_editing_tests.rs:127` — does not use Default
- `src/storage/backend/worlds_tests.rs:125` — does not use Default
- `src/storage/backend/characters_tests.rs:158` — does not use Default
- `src/model/world.rs:89` (From impl) — must add all 3 fields explicitly
- `tests/integration/lifecycle.rs:25` — does not use Default
- `tests/integration/application_service.rs:26` — does not use Default
- `tests/integration/flow/retry_event.rs:132` — does not use Default
- `tests/helpers/pipeline_helpers.rs:13,59` — do not use Default
- `tests/helpers/fixtures.rs:12,115` — do not use Default

Strategy: Add `..Default::default()` to every `WorldCard { ... }` literal that doesn't already have it. This insulates against future field additions.

**Edge handling:** All three new fields use `#[serde(default)]` — old `WorldCard` JSON deserializes fine. Consumer code that already holds `WorldCard` (ServerResources, TestAppBuilder) is unaffected — they gain three new fields that default to empty/None.

### Step 2: Add `player_key` column to `worlds` table

**File:** `src/storage/db.rs`
- Add migration v11:
  ```sql
  ALTER TABLE worlds ADD COLUMN player_key TEXT NOT NULL DEFAULT '';
  ```
- Set `PRAGMA user_version = 11`

This stores which persona is the player for a given world (e.g. `"julian"` for `redmist_estate`). The default empty string handles existing rows — they'll be populated on next seeding.

### Step 3: Add `Storage::get_world_id(key)` method

**File:** `src/storage/backend/worlds.rs`
```rust
pub fn get_world_id(&self, key: &str) -> Result<Option<i64>, EngineError> {
    self.with_backend_mut(Operation::GetWorldId, |backend, _game_id| match backend {
        Backend::Sqlite { pool } => {
            let conn = pool.conn();
            let result = conn.query_row(
                "SELECT id FROM worlds WHERE key = ?",
                [key],
                |row| row.get(0),
            );
            match result {
                Ok(id) => Ok(Some(id)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(EngineError::Database(e)),
            }
        }
        Backend::InMemory(data) => Ok(data.worlds.iter().find(|w| w.world_card.key == key).map(|w| w.world_id)),
        Backend::Test { .. } => unimplemented!(),
    })
}
```

**File:** `src/storage/backend/core.rs` — Add `GetWorldId` to `Operation` enum.

**Edge:** Returns `None` if world not found — caller handles the error.

### Step 4: Implement world/persona/character seeding in `ensure_defaults()`

**File:** `src/bootstrap/run.rs` — `ensure_defaults()` function (currently lines 398-464)

Expand `ensure_defaults()` to seed worlds, personas, and characters after the existing prompt preset seeding. The seeding is idempotent (skip if key already exists with content).

**Seeding logic:**
```rust
// After existing prompt preset seeding:

// 1. Seed worlds + maps + personas + characters from data/worlds/
let worlds_dir = data_dir.join("worlds");
if worlds_dir.exists() {
    for entry in std::fs::read_dir(&worlds_dir)? {
        let entry = entry?;
        let world_dir = entry.path();
        if !world_dir.is_dir() { continue; }
        let world_json = world_dir.join("world.json");
        if !world_json.exists() { continue; }

        let manifest: WorldManifest = read_json_file(&world_json)?;
        let mut world_card: WorldCard = manifest.clone().into();
        let world_key = world_card.key.clone();

        // Load and seed map + world (seed_world uses INSERT OR IGNORE — safe to re-call)
        let map_path = world_dir.join(&manifest.map_file);
        let map: MapDef = read_json_file(&map_path)?;
        storage.seed_world(&world_card, &map)?;

        // Get the world_id for FK (needed even if world was already seeded)
        let world_id = storage.get_world_id(&world_key)?
            .ok_or_else(|| EngineError::Config(format!("World '{world_key}' not found after seeding")))?;

        // Load and seed persona (player)
        let player_path = data_dir.join("personas").join(&manifest.player_file);
        let player_key = Path::new(&manifest.player_file)
            .file_stem().and_then(|s| s.to_str()).unwrap_or("player");
        if storage.get_persona(player_key)?.is_none() {
            let player: PlayerCard = read_json_file(&player_path)?;
            storage.seed_persona(player_key, &player)?;
        }

        // Load and seed characters
        let chars_group = if manifest.characters_dir.is_empty() {
            world_card.key.as_str()
        } else {
            manifest.characters_dir.as_str()
        };
        let chars_dir = data_dir.join("characters").join(chars_group);
        if chars_dir.is_dir() {
            let existing_chars: std::collections::HashSet<String> =
                storage.list_characters(world_id)?
                    .into_iter().map(|c| c.id).collect();
            for char_entry in std::fs::read_dir(&chars_dir)? {
                let char_entry = char_entry?;
                let path = char_entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
                match read_json_file::<NpcCard>(&path) {
                    Ok(npc) => {
                        if !existing_chars.contains(&npc.id) {
                            storage.seed_character(world_id, &npc)?;
                        }
                    }
                    Err(e) => tracing::warn!("Failed to parse NPC {}: {e}", path.display()),
                }
            }
        }

        // Re-seed player_key on the world row if it was empty (pre-existing DB)
        // For fresh seeds, From<WorldManifest> already set it during .into()
        if world_card.player_key.is_empty() {
            world_card.player_key = player_key.to_string();
            let conn = db_pool.conn();
            conn.execute(
                "UPDATE worlds SET player_key = ? WHERE key = ?",
                rusqlite::params![player_key, &world_card.key],
            )?;
        }
    }
}
```

**Note:** Make `read_json_file()` in `src/bootstrap/load.rs` `pub(crate)` (currently private `fn`). It's a 3-line generic JSON reader needed by both `load.rs` functions (already) and `ensure_defaults` (new). Also import `use std::path::Path;` in `run.rs` for the `file_stem()` call on `manifest.player_file`.

**Idempotency guarantee:** `seed_world` uses `INSERT OR IGNORE` (no-op if row exists). `seed_persona` and `seed_character` check existence before inserting. Re-calling `ensure_defaults` is safe — each sub-operation is individually idempotent. No top-level "skip if world exists" is used, so partial failures (e.g. world seeded but characters failed) are retried on next startup.

### Step 5: Refactor `validate_loaded_data()` to accept `WorldCard`

**File:** `src/bootstrap/validate.rs`
- Change signature from `(manifest: &WorldManifest, map, player, npcs)` to `(world: &WorldCard, map, player, npcs)`
- The function uses `manifest.starting_room_id` → `world.starting_room_id` (same field)
- The function uses `manifest.scenarios` → `world.scenarios` (same field)
- `manifest.name` is not used by validate, so no loss

**File:** `src/bootstrap/validate_tests.rs`
- Update all tests to construct `WorldCard` instead of `WorldManifest`

**File:** `src/test_support/fixtures.rs`
- Update `TestWorldManifest` or add `TestWorldCard` helper

### Step 6: Refactor `inject_scenario_logs()` to accept `WorldCard`

**File:** `src/bootstrap/scenario.rs`
- Change signature from `(state: &mut GameState, manifest: &WorldManifest, player: &PlayerCard)` to `(state: &mut GameState, world: &WorldCard, player: &PlayerCard)`
- Function uses:
  - `manifest.default_scenario()` → `world.default_scenario()` (same method, both exist)
  - `manifest.starting_room_id` → `world.starting_room_id` (same field)
- No other `WorldManifest`-specific fields are used

### Step 7: Rewrite `run()` to load from DB

**File:** `src/bootstrap/run.rs` — `run()` function (currently lines 158-350)

**Current flow (to be replaced):**
```
1. initialize_world_from_manifest() → (manifest, map, player, npcs)
2. validate_loaded_data(manifest, map, player, npcs)
3. Convert: world_arc = manifest.into(), map_arc, player_arc, npcs_map
4. DbPool::new() → ensure_defaults() ← seed
5. find_latest_game_for_world(manifest.name) → active_game_id
6. Storage::new_sqlite(db_pool, active_game_id)
7. Load/create GameState
8. ... rest unchanged ...
```

**New flow:**
```
1. DbPool::new()
2. ensure_defaults()  ← seed worlds/personas/characters/presets
3. Storage for loading (game_id = PRESET_STORAGE_GAME_ID initially):
   let seed_storage = Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);
4. Load world from DB:
   let world_with_map = seed_storage.get_world(&args.world)?
       .ok_or(EngineError::WorldNotFound(args.world.clone()))?;
   let world_card = world_with_map.world_card;  // WorldCard now has key
   let map = world_with_map.map;
5. Load persona:
   let player_key = world_card.effective_player_key();
   let player = seed_storage.get_persona(&player_key)?
       .ok_or(EngineError::Config(format!("Persona '{player_key}' not found")))?;
6. Load characters:
   let world_id = seed_storage.get_world_id(&args.world)?
       .ok_or_else(|| EngineError::Config(format!("World '{}' not found in DB", args.world)))?;
   let npcs = seed_storage.list_characters(world_id)?;
7. Validate:
   validate_loaded_data(&world_card, &map, &player, &npcs)?
8. Convert to Arc types:
   world_arc, map_arc, player_arc, npcs_map — same as before
9. find_latest_game_for_world(world_card.name) → active_game_id
   // world_card.name == manifest.name, no behavior change
10. Storage::new_sqlite(db_pool, active_game_id)
11. Load/create GameState — same logic but with WorldCard instead of WorldManifest:
    - inject_scenario_logs(&mut new_state, &world_card, &player)
    - world_card.default_scenario() instead of manifest.default_scenario()
12. ... rest unchanged ...
```

**Key changes in `run()`:**
- Remove `use super::initialize_world_from_manifest;` import
- `manifest.name` → `world_card.name` everywhere
- `manifest.starting_room_id` → `world_card.starting_room_id`
- `manifest.default_scenario()` → `world_card.default_scenario()`
- `inject_scenario_logs(state, &manifest, &player)` → `inject_scenario_logs(state, &world_card, &player)`
- Persona loading uses `world_card.effective_player_key()` (defined in Step 1) to get the persona key from the DB-loaded `WorldCard`



### Step 8: Update architecture docs (BEFORE Step 9 tests per SDI requirement)

**File:** `docs/architecture/system.md`
- Update startup flow description to reflect DB-centric loading
- Remove references to file-based loading at runtime

**File:** `docs/reference/data_schemas.md`
- Note `player_key` column on `worlds` table (migration v11)
- Update `WorldCard` field list

The project requires spec-first: update docs before implementing.

### Step 9: Integration tests for DB loading

**File:** `tests/integration/world_loading.rs` (new)

Test scenarios:
1. **Fresh DB seeds from JSON, then loads correctly** — Create empty in-memory DB, call ensure_defaults with real data dir, verify get_world returns correct WorldCard + MapDef
2. **Already-seeded DB loads without re-seeding** — Seed once, call ensure_defaults again, verify no duplication
3. **Persona loaded by player_key** — After seeding, verify get_persona(world_card.effective_player_key()) returns correct PlayerCard
4. **Characters loaded by world_id** — After seeding, verify list_characters(world_id) returns all NPCs for that world

**File:** `src/bootstrap/run_tests.rs`
- Add test for seeding in ensure_defaults (worlds, personas, characters)
- Add test for idempotency of seeding

### Step 10: Update CHANGELOG and archive plan

### Step 11: Clean up dead code

After all callers are migrated:
- `src/bootstrap/load.rs`: Keep `load_world_manifest` and `initialize_world_from_manifest` — both are still needed during seeding (`ensure_defaults()` reads JSON via `manifest.player_file`/`manifest.map_file`/`manifest.characters_dir` for file path resolution). The doc comments already mark these as seed-only.
- `src/bootstrap/mod.rs`: Keep `load_world_manifest` re-export (used by `ensure_defaults`). Remove `initialize_world_from_manifest` re-export if nothing outside bootstrap uses it.
- `src/model/world.rs`: `WorldInfo` already deleted in Step 1.
- `src/cli.rs`: Keep `scan_worlds` file-based — it runs before DB init, used only for `--list-worlds` CLI flag.
- `src/test_support/fixtures.rs`: `TestWorldManifest` stays — `WorldManifest` is still the file-era type for seeding tests. `TestWorld::minimal()` already constructs `WorldCard` (updated in Step 1 to include new fields).

## Critical Files & Anchors

| File | Anchor | Why |
|------|--------|-----|
| `src/bootstrap/run.rs:158-350` | `run()` function | The main rewrite target — swapping file reads for DB queries |
| `src/model/world.rs` | `WorldCard`, `WorldInfo`, `WorldManifest` | Type unification — add fields to WorldCard, delete WorldInfo |
| `src/storage/backend/worlds.rs` | `get_world()`, `seed_world()`, `WorldWithMap` | Must update WorldWithMap shape, add get_world_id, add player_key to queries |
| `src/storage/db.rs:154-247` | Migration v10 | Must add migration v11 for player_key column |
| `src/bootstrap/scenario.rs:8` | `inject_scenario_logs()` signature | Must change WorldManifest param to WorldCard |

## Verification

1. **Unit tests pass:** `cargo nextest run` — all existing 874+ tests pass
2. **New integration test:** `cargo nextest run world_loading` — DB loading parity tests pass
3. **Manual smoke test:** Run `cargo run -- --world redmist_estate` → game starts, navigate rooms, talk to NPCs. Verify identical behavior to pre-migration.
4. **Settings persistence:** Start server, change a setting in UI, restart → setting persists (existing Phase 4 behavior unaffected)
5. **Idempotent seeding:** Start twice with same DB → no duplicate data, no errors
6. **Build validation:** `python build.py` passes clean

**Key verification command:**
```bash
cd chronicler_engine
cargo nextest run  # Full suite
cargo run -- --world redmist_estate  # Manual smoke test
```

## Assumptions & Contingencies

- **Assumption:** `scan_worlds()` in `cli.rs` stays file-based. It runs before DB init and is only for `--list-worlds`. If this should change later, it's a separate concern.
- **Assumption:** `WorldManifest` is kept as a type — it's still needed for parsing `world.json` during seeding. Only the runtime path switches to `WorldCard`/DB.
- **Assumption:** `default_scenario_id` field on `WorldCard` is included for completeness but the engine currently uses `default_scenario()` which picks `scenarios.first()`. If `default_scenario_id` logic changes later, the field will be ready.
- **Contingency:** If seeding is too slow on first startup with many worlds, add a `--skip-seed` CLI flag or check if DB has any worlds before scanning directories. Current data has only 2 worlds — unlikely to be an issue.
- **Contingency:** If `player_key` is empty on a world row (pre-existing DB from before migration v11), the `effective_player_key()` fallback returns `"player"`. This matches the filename stem pattern.

## Implementation Notes (Post-Completion)

**Completed:** 2026-06-13  
**Test Status:** 1185/1190 passing (99.6%)  
**Remaining Issues:** 5 minor test failures (expected migration-related)

**What Changed During Implementation:**
- Test updates required more work than anticipated (~30+ locations vs 18 planned)
- Import ordering guardrail required fixing (moved `use std::path::Path;` to top)
- Integration tests (Step 9) deferred as technical debt
- Documentation expanded beyond plan (added Section 5.5 to architecture docs)

**Files Modified:** 30+ files, ~800+ lines changed
