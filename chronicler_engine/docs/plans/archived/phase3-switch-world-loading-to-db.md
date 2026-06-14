# Plan: Phase 3 - Switch Runtime World Loading from Files to Database

## Context

This plan completes the game data migration started in `db-game-data-migration.md`. Phases 1, 2, and 4 are complete:
- ✅ Phase 1: Schema + Storage CRUD (DB tables + backend modules)
- ✅ Phase 2: Seed Logic (JSON → DB at startup)
- ⏸️ **Phase 3: Switch Reads to DB** (THIS PLAN)
- ✅ Phase 4: Settings Write-Through (DB-backed settings persistence)

## Problem Statement

Currently, `bootstrap/run.rs` still loads world data from JSON files via `initialize_world_from_manifest()`, even though the data exists in the database after seeding. This creates:
- **Redundant I/O**: Files read even though DB has the data
- **Inconsistency risk**: File and DB could diverge if manually edited
- **Missing capability**: Can't verify DB loading works correctly

## Goal

Replace file-based world loading with DB queries in `bootstrap/run.rs`. After this change:
- JSON files are **seed-only** (never read at runtime after migration)
- All world data (worlds, maps, personas, characters) loaded from DB
- Game plays identically to file-based loading
- All 874+ tests continue passing

## Scope

### In Scope
- Modify `bootstrap/run.rs` lines ~165-200 to use DB queries
- Replace `initialize_world_from_manifest()` with `storage.get_world()`
- Replace persona/character file reads with `storage.get_persona()` and `storage.list_characters()`
- Update scenario injection to work with DB-seeded data
- Update validation logic for DB-loaded data
- Integration tests verifying DB loading parity

### Out of Scope
- Changing `ServerResources` struct (already accepts model types)
- Modifying action pipeline or game service (consume same model types)
- UI changes (separate UI CRUD plan)
- Model refactoring (`WorldManifest` vs `WorldCard` convergence)

## Implementation Steps

### Step 1: Update `run()` function signature and flow

**File:** `src/bootstrap/run.rs`

**Current:**
```rust
let (manifest, map, player, npcs) = initialize_world_from_manifest(&args.world, &data_dir)?;
// ... validation ...
let db_pool = crate::storage::db::DbPool::new(db_path)?;
ensure_defaults(&db_pool, &data_dir)?;
```

**Target:**
```rust
let db_pool = crate::storage::db::DbPool::new(db_path)?;
ensure_defaults(&db_pool, &data_dir)?; // Seeds if needed

// Load from DB (not files)
let storage = crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);
let world_result = storage.get_world(&args.world)?;
let (world_with_map, world_key) = match world_result {
    Some(w) => (w, w.world_key.clone()),
    None => { /* error: world not found in DB */ }
};

// Extract data
let world_card = Arc::new(world_with_map.world);
let map_arc = Arc::new(world_with_map.map);
let player_arc = Arc::new(/* load from storage.get_persona() */);
let npcs_map = /* load from storage.list_characters(world_id) */;
```

### Step 2: Handle world_id for character loading

**Challenge:** `storage.list_characters()` needs `world_id` (numeric PK), but we only have `world_key` (string).

**Solution:** Query world_id after loading world:
```rust
let db_conn = db_pool.conn();
let world_id: i64 = db_conn.query_row(
    "SELECT id FROM worlds WHERE key = ?",
    [&args.world],
    |row| row.get(0)
)?;
let npc_list = storage.list_characters(world_id)?;
```

### Step 3: Replace validation

**Current:** `validate_loaded_data(&manifest, &map, &player, &npcs)`

**Target:** Since seeding already validated data integrity, use lighter validation:
```rust
// Verify critical invariants
assert!(valid_room_ids_exist(&map_arc, &world_card.starting_room_id));
validate_npc_triggers(&map_arc, &npcs_vec)?; // Ensure trigger room_ids exist
```

### Step 4: Handle scenario injection

**Challenge:** `inject_scenario_logs()` expects `WorldManifest`, but we have `WorldCard`.

**Options:**
1. **Convert `WorldCard` → `WorldManifest`** (temporary, adds duplication)
2. **Refactor `inject_scenario_logs()` to accept `WorldCard`** (cleaner)
3. **Store scenarios in `WorldWithMap` struct** (best, already in plan)

**Recommended:** Option 3 - `WorldWithMap` already includes `scenarios` field from plan.

### Step 5: Update imports and remove unused code

**Remove:**
- `use super::initialize_world_from_manifest;`
- `use super::validate_loaded_data;` (if no longer used)

**Add:**
- Storage imports as needed

### Step 6: Test integration

**Test scenarios:**
1. **Empty DB**: First startup, seed from JSON → load from DB ✅
2. **Seeded DB**: Restart, skip seeding → load from DB ✅
3. **Modified DB**: Edit setting in UI → restart → change persists ✅
4. **Parity**: Game plays identically before/after migration ✅

**Test locations:**
- `tests/integration/world_loading.rs` (new file)
- `src/bootstrap/run_tests.rs` (existing, add DB loading tests)

## Files to Modify

| File | Change |
|------|--------|
| `src/bootstrap/run.rs` | Replace file loading with DB queries (~35 lines changed) |
| `src/bootstrap/load.rs` | Keep for backward compat during transition |
| `tests/integration/world_loading.rs` | NEW: Integration tests for DB loading |
| `src/bootstrap/run_tests.rs` | Add 2-3 tests for DB loading paths |

## Testing Strategy

### Unit Tests
- `storage.get_world()` returns correct structure
- `storage.list_characters(world_id)` filters by world
- `storage.get_persona(key)` loads player card

### Integration Tests
```rust
#[test]
fn test_world_loaded_from_db_after_seeding() {
    // Setup: Empty DB
    // Act: Call run() with world key
    // Assert: World loaded from DB (not files)
}

#[test]
fn test_game_state_initialized_correctly_from_db() {
    // Setup: Seed DB with known world
    // Act: Initialize game
    // Assert: GameState has correct rooms, npcs, starting position
}
```

### Regression Testing
- Run full test suite (874+ tests) - all must pass
- Browser tests: Navigate world, talk to NPCs, verify behavior unchanged
- Manual playthrough: Redmist Estate works identically

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scenario injection breaks | High | Test with all scenarios, verify logs match |
| FK constraint violations | Medium | Ensure world_id queried correctly before loading characters |
| Persona key mismatch | Medium | Use consistent key format (filename stem or worldkey_player) |
| Performance regression | Low | DB queries faster than file I/O, measure if concerned |

## Verification Checklist

- [ ] All 874+ tests pass
- [ ] Coverage remains ≥80% for modified files
- [ ] Clippy clean (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Import ordering guardrail passes
- [ ] Doc anchors added for new public functions
- [ ] Integration tests verify DB loading
- [ ] Manual playthrough: game behaves identically
- [ ] Settings persist across restarts
- [ ] Plan document created and archived after completion

## Acceptance Criteria

1. **Functionality**: `run()` loads all world data from DB (not files)
2. **Parity**: Game plays identically to file-based loading
3. **Tests**: All existing tests pass + 2-3 new integration tests for DB loading
4. **Coverage**: Modified files maintain ≥80% coverage
5. **Documentation**: `docs/architecture/system.md` updated with final state

## Timeline Estimate

- **Implementation**: 2-4 hours (careful refactoring + testing)
- **Verification**: 1-2 hours (integration tests + manual testing)
- **Total**: 3-6 hours

**Note:** This is a focused, well-scoped refactor. Storage infrastructure already exists (Phases 1, 2, 4 complete). This is "last mile" work to switch the runtime loading path.

## Related Plans

- **Completed:** `docs/plans/archived/db-game-data-migration.md` (Phases 1, 2, 4)
- **Future:** UI CRUD implementation for world/character management
