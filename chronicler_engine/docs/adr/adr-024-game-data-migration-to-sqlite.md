# ADR 0010: Migrate Game Data to SQLite with Seed Pattern

**Date:** 2026-06-01  
**Status:** Accepted (Phases 1, 2, 4 implemented; Phase 3 deferred)  
**Drivers:** UI CRUD requirements, settings persistence

## Problem Statement

The Chronicler Engine stored all game data (worlds, maps, characters, personas, settings) as JSON files read at startup. This created several limitations:

1. **No UI-driven CRUD**: Creating/editing worlds or characters required manual file editing
2. **No change tracking**: JSON files couldn't track who made changes or when
3. **No relational integrity**: Cross-references between entities weren't validated
4. **File I/O on every read**: Settings file was read/written multiple times per session

## Decision

Migrate game data to SQLite database using a **seed pattern**:

### Pattern

- JSON files act as **seed templates** only
- At startup: read JSON files → insert into DB (if not already present)
- At runtime: **DB is sole source of truth**
- JSON files never written back to

### Schema Design

| Table | Approach |
|-------|----------|
| `worlds` | Merged WorldManifest+WorldCard, `key` column preserves original string ID |
| `maps` | Single JSON blob (`MapDef`), 1:1 with worlds |
| `personas` | PlayerCard flattened with JSON blobs for nested arrays |
| `characters` | NpcCard with FK to worlds, triggers/relationships as JSON blobs |
| `settings` | Singleton row (id=1), all fields JSON blobs for arrays |

**JSON blobs** used for nested collections (triggers, relationships, scenarios, connections, agents) to avoid over-normalization while maintaining queryability at the entity level.

### Implementation Phases

1. **Phase 1: Schema + Storage CRUD** — ✅ COMPLETE
   - Migration v10 with 5 new tables
   - Backend CRUD modules for all entity types
   - InMemory backend support for tests

2. **Phase 2: Seed Logic** — ✅ COMPLETE
   - `ensure_defaults()` expanded to seed all entities
   - Idempotent seeding (skip if key exists with content)
   - Proper FK handling

3. **Phase 3: Switch Reads to DB** — ⏸️ DEFERRED
   - Requires refactoring `bootstrap/run.rs`
   - Deferred until UI CRUD implementation
   - Current state: file reads still used for world loading

4. **Phase 4: Settings Write-Through** — ✅ COMPLETE
   - `AppSettings::save()` → DB write
   - `load_settings()` → DB read
   - All UI handlers persist automatically

## Consequences

### Positive

- ✅ **UI CRUD enabled**: Storage layer ready for world/character management UI
- ✅ **Settings persistence**: Changes persist across restarts automatically
- ✅ **Change tracking**: `created_at`/`updated_at` on all rows
- ✅ **Relational integrity**: FK constraints prevent orphan records
- ✅ **Test support**: InMemory backend maintains test isolation

### Negative

- ⚠️ **Phase 3 deferred**: World loading still file-based until bootstrap refactored
- ⚠️ **Migration complexity**: Requires careful schema evolution for future changes
- ⚠️ **DB file management**: Users must manage SQLite files (backups, migrations)

### Neutral

- JSON files remain as seed templates (backward compatible)
- Settings still serializable to JSON for export/import
- InMemory backend adds ~200 lines to Storage tier

## Files Changed

- `src/storage/db.rs` — Migration v10
- `src/storage/models/` — New DB row structs
- `src/storage/backend/` — New CRUD modules (worlds, personas, characters, settings)
- `src/bootstrap/run.rs` — Seed logic in `ensure_defaults()`
- `src/settings.rs` — DB-backed settings persistence
- `docs/architecture/system.md` — Storage tier documentation
- `docs/reference/data_schemas.md` — Database schema documentation

## Related

- ADR 0009: Unified Storage Backend (prerequisite for this migration)
- Future: UI CRUD implementation for game data management
