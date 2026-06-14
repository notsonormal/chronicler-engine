# Plan: Phase 3 Runtime Loading Fix

## Context

The Phase 3 DB migration (ADR-024) added seeding infrastructure but left the runtime loading path untouched. `bootstrap/run.rs` still calls `initialize_world_from_manifest()` which reads JSON files, contradicting the architecture docs claiming "100% DB-first at runtime". This plan fixes the runtime path to actually load from SQLite.

## Approach

### Step 1: Update `run()` to load world from DB after seeding

**File:** `src/bootstrap/run.rs`

**Current (lines 165-178):**
```rust
let data_dir = resolve_engine_data_path();
let (manifest, map, player, npcs) = initialize_world_from_manifest(&args.world, &data_dir)?;

let world_card: crate::model::world::WorldCard = manifest.clone().into();
if let Err(e) = validate_loaded_data(&world_card, &map, &player, &npcs) { ... }

let world_arc = Arc::new(manifest.clone().into());
let map_arc = Arc::new(map);
let player_arc = Arc::new(player.clone());
let npcs_map: HashMap<_, _> = npcs.into_iter().map(|n| (n.id.clone(), n)).collect();
```

**Target:**
```rust
let data_dir = resolve_engine_data_path();
let db_dir = std::env::current_exe()
    .ok()
    .and_then(|p| p.parent().map(std::path::PathBuf::from))
    .unwrap_or_else(|| data_dir.clone());
let db_path = db_dir.join(format!("chronicler_{}.db", args.port));

let db_pool = crate::storage::db::DbPool::new(db_path.to_str().unwrap_or("chronicler.db"))?;
ensure_defaults(&db_pool, &data_dir)?; // Seeds if needed

let storage = crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);

// Load world from DB (not files)
let world_result = storage.get_world(&args.world)?;
let (world_with_map, world_id) = match world_result {
    Some(w) => (w, w.world_id),
    None => {
        tracing::error!("World '{}' not found in database", args.world);
        eprintln!("World '{}' not found in database. Check that world exists.", args.world);
        std::process::exit(1);
    }
};

let world_card = Arc::new(world_with_map.world_card);
let map_arc = Arc::new(world_with_map.map);

// Load player persona from DB
let player_key = world_card.effective_player_key();
let player: PlayerCard = match storage.get_persona(player_key)? {
    Some(p) => p,
    None => {
        tracing::error!("Persona '{}' not found in database", player_key);
        eprintln!("Persona '{}' not found in database.", player_key);
        std::process::exit(1);
    }
};
let player_arc = Arc::new(player);

// Load characters from DB
let npcs_vec = storage.list_characters(world_id)?;
let npcs_map: HashMap<_, _> = npcs_vec.into_iter().map(|n| (n.id.clone(), n)).collect();

// Validate loaded data (now from DB)
if let Err(e) = validate_loaded_data(&world_card, &map_arc, &player_arc, &npcs_vec) {
    tracing::error!("Data validation failed for world '{}':\n{}", args.world, e);
    eprintln!("Data validation failed for world '{}':\n{}", args.world, e);
    std::process::exit(1);
}
```

**Cleanup:** Remove `initialize_world_from_manifest` import. Keep `validate_loaded_data`.

### Step 2: Replace `manifest.name` with `world_card.name` for game lookup

**File:** `src/bootstrap/run.rs` (lines 192-211)

**Current:**
```rust
let active_game_id = match find_latest_game_for_world(&db_pool, &manifest.name)? {
    Some((id, name)) => { ... }
    None => {
        let existing_names = list_game_names_for_world(&db_pool, &manifest.name)?;
        let name = generate_game_name(&manifest.name, &existing_names);
        let conn = db_pool.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            rusqlite::params![&manifest.name, &name, &now],
        )?;
        ...
    }
};
```

**Target:** Replace all `&manifest.name` with `&world_card.name`:
```rust
let active_game_id = match find_latest_game_for_world(&db_pool, &world_card.name)? {
    // ... rest unchanged except manifest.name → world_card.name
```

### Step 3: Replace `manifest.default_scenario()` with `world_card.default_scenario()`

**File:** `src/bootstrap/run.rs` (line 241)

**Current:**
```rust
if let Some(scenario) = manifest.default_scenario() {
    new_state.init_scenario_npcs(scenario);
}
```

**Target:**
```rust
if let Some(scenario) = world_card.default_scenario() {
    new_state.init_scenario_npcs(scenario);
}
```

### Step 4: Optimize `seed_game_data()` to use `get_world_id()` instead of `get_world()`

**File:** `src/bootstrap/load.rs` (lines 105-116)

**Current:**
```rust
let world_id = match storage.get_world(&world_key)? {
    Some(existing) => existing.world_id,
    None => {
        let world_card: WorldCard = manifest.clone().into();
        let map_path = world_dir.join(&manifest.map_file);
        let map: MapDef = read_json_file(&map_path)?;
        storage.seed_world(&world_card, &map)?;
        storage.get_world_id(&world_key)?.ok_or_else(|| ...)?
    }
};
```

**Target:**
```rust
let world_id = match storage.get_world_id(&world_key)? {
    Some(id) => id,
    None => {
        let world_card: WorldCard = manifest.clone().into();
        let map_path = world_dir.join(&manifest.map_file);
        let map: MapDef = read_json_file(&map_path)?;
        storage.seed_world(&world_card, &map)?;
        storage.get_world_id(&world_key)?.ok_or_else(|| {
            EngineError::Config(format!("World '{world_key}' not found after seeding"))
        })?
    }
};
```

### Step 5: Fix `empty_to_none` on `player_key` in `seed_world`

**File:** `src/storage/backend/worlds.rs` (line ~144 in seed_world INSERT)

**Current:**
```rust
empty_to_none(world_card.default_scenario_id.as_deref().unwrap_or("")),
empty_to_none(world_card.default_room_image.as_deref().unwrap_or("")),
&world_card.player_key,
```

**Target:** Remove `empty_to_none` wrapper from `player_key`:
```rust
empty_to_none(world_card.default_scenario_id.as_deref().unwrap_or("")),
empty_to_none(world_card.default_room_image.as_deref().unwrap_or("")),
&world_card.player_key,  // player_key is non-optional String, insert as-is
```

### Step 6: Update test fixtures to use `..Default::default()` consistently

**File:** `src/bootstrap/validate_tests.rs`

**Lines 10-20, 31-41, 60-70:** Replace explicit `WorldCard { key: "...", name: "...", starting_room_id: "...", description: "...", global_rules: vec![], scenarios: vec![], default_scenario_id: None, default_room_image: None, player_key: "" }` with:

```rust
let world = WorldCard {
    key: "test".to_string(),
    name: "Test".to_string(),
    starting_room_id: "room_a".to_string(),
    player_key: "player".to_string(),
    ..Default::default()
};
```

Same pattern for lines 128-155 (`test_validate_loaded_data_multiple_errors`).

### Step 7: Clean up duplicate `manifest.clone().into()` call

**File:** `src/bootstrap/run.rs` (line 175)

**Current:**
```rust
let world_card: crate::model::world::WorldCard = manifest.clone().into();
// ...
let world_arc = Arc::new(manifest.clone().into());
```

**Target:**
```rust
let world_card = crate::model::world::WorldCard = /* from DB */;
// ...
let world_arc = Arc::new(world_card.clone());
```

This eliminates the second manifest clone+convert.

## Critical Files & Anchors

| File | Region | Reason |
|------|--------|--------|
| `src/bootstrap/run.rs` | lines 159-260 | `run()` function — main runtime loading path to rewrite |
| `src/bootstrap/load.rs` | lines 81-159 | `seed_game_data()` — optimize to use `get_world_id()` |
| `src/storage/backend/worlds.rs` | lines 135-193 | `seed_world()` — remove unnecessary `empty_to_none` on `player_key` |
| `src/bootstrap/validate_tests.rs` | all `WorldCard` constructions | Test fixture consistency |

## Verification

### Build Check
```bash
cd chronicler_engine
cargo check --all-features
```

### Test Suite
```bash
cd chronicler_engine
python build.py
```
Expected: All tests pass (1190+ per changelog claim).

### Integration Test: Fresh DB Startup
```bash
cd chronicler_engine
cargo run -- --world redmist_estate
```
Expected: Seeds world from JSON files on first startup, loads from DB, game starts normally.

### Integration Test: Restart with Seeded DB
```bash
cd chronicler_engine
cargo run -- --world redmist_estate
```
Expected: Skips seeding (data exists), loads entirely from DB, game state preserved.

### Verify No File I/O at Runtime
After startup with existing DB, rename/move the `data/worlds/redmist_estate/` directory:
```bash
mv chronicler_engine/data/worlds/redmist_estate chronicler_engine/data/worlds/redmist_estate.bak
cargo run -- --world redmist_estate
```
Expected: Game starts successfully (proves runtime uses DB, not files).

## Assumptions & Contingencies

- **Assumption:** `--list-worlds` CLI flag should continue using file-based scan (runs before DB init). No change needed to `cli.rs::scan_worlds`.
- **Assumption:** `WorldManifest` stays for seeding only — no runtime usage after this fix.
- **If DB is corrupt/empty:** `get_world()` returns `None`, error message prompts user to check world exists or delete DB to re-seed.
- **If `get_world_id()` returns None after seeding:** This is a bug — `seed_world` does INSERT OR IGNORE then queries back. Error path already handles this.
