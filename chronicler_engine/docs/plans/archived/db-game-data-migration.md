# Plan: Migrate Game Data to Database with Seed Pattern

## Goal

Move worlds, maps, personas, characters, and settings from file-based loading to SQLite database. JSON files become seed data only (read once on first startup if DB row doesn't exist, then never again). The DB is the sole source of truth at runtime.

This directly enables future UI-driven CRUD for game data.

## Decisions (from grilling session)

| Decision | Choice |
|---|---|
| **Problem being solved** | Enable UI CRUD for game data; DB is prerequisite |
| **JSON files after seeding** | Seed-only, never written back to |
| **Settings** | Also move to DB (same pattern as everything else) |
| **Data scope in DB** | Global (like prompt presets), shared across all games |
| **Maps** | Single `maps` table with full MapDef as JSON blob (rooms, regions, overworld all embedded) |
| **Overworld/Region** | No separate table — nested inside maps JSON blob |
| **Triggers on characters** | JSON blob column on `characters` table |
| **Other nested collections** | JSON blob columns (global_rules, scenarios, relationships, character sheet fields, whole map structure) |
| **Seed timing** | All at application startup (like prompt presets) |
| **WorldManifest/WorldCard** | Merge into single `worlds` table row |
| **Primary keys** | Auto-increment integer `id` as PK; original string identifiers (e.g. `redmist_estate`) stored as `key` column for lookups and LLM context |

## Implementation Status

### Phase 1: Schema + Storage CRUD ✅ COMPLETE
- Migration v10 added to `db.rs`
- DB row structs in `src/storage/models/` for all 5 tables
- 4 storage backend modules implemented
- `InMemoryData` extended with new collections
- Operation enum extended with 12 new variants
- All unit tests passing

### Phase 2: Seed Logic ✅ COMPLETE
- `ensure_defaults()` in `bootstrap/run.rs` expanded
- Seeds all 5 entity types from JSON files
- Idempotent seeding (skip if key exists)
- Proper FK handling for characters

### Phase 3: Switch Reads to DB ⏸️ DEFERRED
**Deferred Reason**: Requires careful refactoring of bootstrap/run.rs. Storage infrastructure is complete - can be done when UI CRUD is implemented.

### Phase 4: Settings Write-Through ✅ COMPLETE
- `AppSettings::save()` migrated to DB write
- `load_settings()` reads from DB
- All settings_fragment handlers updated
- Tests updated for DB-backed behavior

## Verification

1. ✅ All 874 tests passing
2. ✅ Settings persist to DB
3. ✅ Seed logic idempotent
4. ⏸️ Runtime world loading still file-based (Phase 3 deferred)
