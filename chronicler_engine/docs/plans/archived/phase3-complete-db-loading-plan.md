# Plan: Complete Phase 3 — Switch `run()` to DB-First Loading

## Context

The current uncommitted Phase 3 diff adds `seed_game_data()` and updates model/storage types, but `run()` still loads world data from JSON files via `initialize_world_from_manifest()`. The architecture docs already describe a seed-once/load-from-DB pattern that isn't implemented in the runtime path. This plan replaces the file I/O in `run()` with DB queries, completing the migration so that JSON files are seed-only.

## Approach

### Step 1: Replace `initialize_world_from_manifest()` call in `run()` with DB-first loading

**File:** `src/bootstrap/run.rs`

**Current code (lines 165–175):**
```rust
let (manifest, map, player, npcs) = initialize_world_from_manifest(&args.world, &data_dir)?;
let world_card: crate::model::world::WorldCard = manifest.clone().into();
if let Err(e) = validate_loaded_data(&world_card, &map, &player, &npcs) { ... }
let world_arc = Arc::new(manifest.clone().into());
let map_arc = Arc::new(map);
let player_arc = Arc::new(player.clone());
```

**Replace with:**
```rust
let db_pool = crate::storage::db::DbPool::new(db_path.to_str().unwrap_or("chronicler.db"))?;
if let Err(e) = ensure_defaults(&db_pool, &data_dir) {
    tracing::warn!("Failed to seed game data: {e}");
}
let seed_storage = crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);

let world_with_map = seed_storage.get_world(&args.world)?
    .ok_or_else(|| EngineError::WorldNotFound(args.world.clone()))?;
let world_card = world_with_map.world_card;
let map = world_with_map.map;
let world_id = world_with_map.world_id;

let player_key = world_card.effective_player_key();
let player = seed_storage.get_persona(player_key)?
    .ok_or_else(|| EngineError::Config(format!("Persona '{player_key}' not found")))?;
let npcs = seed_storage.list_characters(world_id)?;
```

**Key changes:**
- Remove `initialize_world_from_manifest()` call entirely
- Move `db_pool` creation and `ensure_defaults()` call **before** the data-loading lines (currently they come after lines 166–175)
- Validate with `validate_loaded_data(&world_card, &map, &player, &npcs)` — signature unchanged
- Load persona via `seed_storage.get_persona(world_card.effective_player_key())`
- Load characters via `seed_storage.list_characters(world_id)` — `world_id` comes from `world_with_map.world_id`
- Error: world not found → `EngineError::WorldNotFound`. Persona not found → `EngineError::Config`. Character list empty is not an error.

**Handles:** `manifest.name` references on lines 192/198/199/204 → replace with `world_card.name`. `manifest.default_scenario()` on line 241 → replace with `world_card.default_scenario()`. The `world_arc` assignment becomes `Arc::new(world_card)` (no double-convert from manifest). `map_arc` becomes `Arc::new(map)`. `player_arc` becomes `Arc::new(player)`.

**After this step:** `run()` does zero file I/O for world/player/NPC data. `initialize_world_from_manifest` is no longer called from `run()`.

### Step 2: Remove `initialize_world_from_manifest` import in `run.rs`

**File:** `src/bootstrap/run.rs` line 22 — remove `use super::load::initialize_world_from_manifest;`

**Keep:** `initialize_world_from_manifest` and `load_world_manifest` in `load.rs` itself — they are still used by `load_tests.rs` (9 tests) and by `seed_game_data()` internally. The `pub(crate)` visibility is correct — these are seed-only utilities.

### Step 3: Remove the now-redundant `db_pool` creation and `ensure_defaults()` call

**File:** `src/bootstrap/run.rs`

The old code had `db_pool` creation and `ensure_defaults()` on lines ~186–194 (after the file load). Step 1 moves these **above** the DB-load lines. Delete the now-duplicated block.

The `db_dir` computation (lines ~183–185) stays where it is — it's used for `db_path` which is now needed earlier.

### Step 4: Clean up `manifest.name` → `world_card.name` in game creation code

**File:** `src/bootstrap/run.rs`, approximately lines 192–207

Replace all `manifest.name` references with `world_card.name`:
- Line 192: `find_latest_game_for_world(&db_pool, &manifest.name)?` → `&world_card.name`
- Line 198: `list_game_names_for_world(&db_pool, &manifest.name)?` → `&world_card.name`
- Line 199: `generate_game_name(&manifest.name, &existing_names)` → `&world_card.name`
- Line 204: `rusqlite::params![&manifest.name, &name, &now]` → `&world_card.name`

### Step 5: Replace `manifest.default_scenario()` with `world_card.default_scenario()`

**File:** `src/bootstrap/run.rs` line 241

Replace:
```rust
if let Some(scenario) = manifest.default_scenario() {
```
with:
```rust
if let Some(scenario) = world_card.default_scenario() {
```

This is the last remaining reference to `manifest` in `run()`.

### Step 6: Remove `effective_player_key()` — make `WorldCard::Default` set `player_key: "player"`

**File:** `src/model/world.rs`

Currently `Default` produces `player_key: String::default()` (empty string), and `effective_player_key()` falls back to `"player"` when empty.

**Change:** Set `player_key: "player".to_string()` in the `Default` impl. Then `effective_player_key()` becomes unnecessary — callers can read `.player_key` directly.

**Update Step 1 code:** Change `seed_storage.get_persona(world_card.effective_player_key())` to `seed_storage.get_persona(&world_card.player_key)`.

**Delete:** `effective_player_key()` method from `WorldCard` impl block.

**Update docs:** Remove `effective_player_key()` references from:
- `docs/architecture/system.md` line 192
- `docs/system/startup.md` line 27
- `docs/reference/data_schemas.md` line 100

**Note:** `derive_player_key()` in `world.rs` already returns `"player"` for empty input — so seeding always populates `player_key` with a non-empty value. The `Default` change just aligns the default state with the seeded state.

### Step 7: Fix `seed_game_data` error handling — make world manifest parse errors non-fatal

**File:** `src/bootstrap/load.rs`, `seed_game_data()` function

Currently: world.json parse failure (line 104) returns `?` which aborts the entire seeding loop. But NPC parse failures (line 152) are `tracing::warn!`.

**Change:** Wrap the per-world manifest parse in a match, skip the world on error, continue to next:

```rust
let manifest: WorldManifest = match read_json_file(&world_json) {
    Ok(m) => m,
    Err(e) => {
        tracing::warn!("Failed to parse world manifest {}: {e}", world_json.display());
        continue;
    }
};
```

**Rationale:** One corrupt world.json should not prevent other worlds from being seeded. This matches the NPC parse behavior already in place.

### Step 8: Rename `WorldSeed` → `InMemoryWorld` in `core.rs`

**File:** `src/storage/backend/core.rs`

**Change:** `pub struct WorldSeed` → `pub struct InMemoryWorld`. The struct holds the full in-memory world record, not a seed.

**Callers to update:**
- `core.rs`: struct definition + `InMemoryData.worlds: Vec<WorldSeed>` → `Vec<InMemoryWorld>`
- `worlds.rs`: `use crate::storage::backend::{..., WorldSeed}` → `InMemoryWorld`; all `WorldSeed { ... }` construction → `InMemoryWorld { ... }`
- All references in `core.rs` `Operation::SeedWorld` dispatch branches

**Clean cutover:** no aliases. Replace all occurrences.

## Critical Files & Anchors

1. **`src/bootstrap/run.rs`** — `run()` function (lines 159–300) — the main function being rewritten. Line 166 `initialize_world_from_manifest` must be replaced. Lines 192–204 `manifest.name`. Line 241 `manifest.default_scenario()`.
2. **`src/bootstrap/load.rs`** — `seed_game_data()` (lines 81–159) — error handling fix on line 104.
3. **`src/model/world.rs`** — `WorldCard` Default impl (line ~65) — change `player_key` default. Delete `effective_player_key()` (line ~105).
4. **`src/storage/backend/core.rs`** — `WorldSeed` struct definition (line ~45) — rename to `InMemoryWorld`.
5. **`src/storage/backend/worlds.rs`** — `WorldSeed` construction and imports — must be renamed.

## Verification

1. **Build check:** `cd chronicler_engine && cargo check` — must pass with zero errors
2. **Clippy:** `cargo clippy --all-targets --all-features -- -D warnings` — must pass
3. **Full test suite:** `python build.py` — all ~1190 tests must pass
4. **Smoke test for the NEW behavior:** Run the engine with `--world redmist_estate`. Verify:
   - Engine starts without error
   - World data appears in the UI (rooms, NPCs, player)
   - `tracing::info!` logs show "Seeded world: redmist_estate" on first run
   - On second run (DB already seeded), no "Seeded" logs appear and game still loads correctly
5. **Unit test verification:** The `seed_game_data` error handling change (Step 7) means a corrupt world.json skips that world. Verify existing `load_tests.rs` tests still pass.

## Assumptions & Contingencies

- **Assumption:** `ensure_defaults()` (which calls `seed_game_data()`) is idempotent and safe to call before data loading. Confirmed: `seed_game_data` uses `get_world()` checks and INSERT OR IGNORE.
- **Assumption:** `Storage::get_persona()` and `Storage::list_characters()` return the same data shapes that file loading would produce. This is the core parity guarantee — the seeded data must match what `initialize_world_from_manifest` would return.
- **Assumption:** `db_path` can be computed before `ensure_defaults` is called. It currently is — `db_dir` depends on `current_exe()` or `data_dir`, both available at function start.
- **Contingency:** If `world_card.player_key` is empty in a pre-migration DB (before v11), the `Default` change means the fallback is `"player"`. But `seed_game_data` always populates `player_key` via `derive_player_key`, so this should only affect hand-crafted DB rows. If it's a concern, keep `effective_player_key()` and skip Step 6.
