# Storage System

**Module:** `crate::adapters::driven::storage`
**Status:** Implemented
**Related ADRs:** ADR-019 (deleted, superseded by ADR-020),
[ADR-020](../adr/adr-020-storage-consolidation.md),
[ADR-024](../adr/adr-024-game-data-migration-to-sqlite.md)

## Overview

The Chronicler Engine storage layer provides unified persistence for game sessions, narrative content, user configurations, and LLM call forensics. It uses a concrete `Storage` struct with a `Backend` enum (`Sqlite`, `InMemory`) for real backends plus a `LayeredBackend` decorator (`Direct(Backend)` | `Test { base, overrides }`) for failure injection, instead of trait-based repository patterns.

**Key Design Decisions:**

- **Unified Storage struct** — Single `Storage` struct with all CRUD operations as methods
- **Backend enum abstraction** — `Sqlite` (production), `InMemory` (dev), `Test` (testing)
- **No trait boilerplate** — No `dyn Trait`, `Arc<dyn>`, or custom mocks
- **Single table per method** — Each storage method touches exactly one table
- **Cross-table coordination in application tier** — `GameServiceContext` composes multi-table operations

## Backend Enum Pattern

`Storage` is a concrete struct, not a trait. `Storage` holds a `Mutex<LayeredBackend>`:

- **`Backend` enum** — real backends only: `Sqlite` (production) and `InMemory` (dev)
- **`LayeredBackend` enum** — `Direct(Backend)` for plain use, or `Test { base: Box<Backend>, overrides }` for wrapping any real backend with failure injection
- `Test` is a decorator, not a peer of `Sqlite`/`InMemory` — `with_backend_mut` unwraps any Test layer before dispatching to the real backend
- Non-recursive `Box<Backend>` (not `Box<LayeredBackend>`) structurally enforces "at most one Test layer" (replace-not-nest invariant)

**Contracts:**

- Game-scoped operations use `game_id` set at construction (`Storage::new_sqlite(pool, game_id)`), not passed per-call
- Each method touches exactly one table. Cross-table coordination happens in the application tier (`src/application/context.rs`)
- All queries use parameterized bindings — no SQL injection surface
- For constructor signatures and method listings, see `src/adapters/driven/storage/backend/core.rs` and the per-table modules

## Schema

Schema is defined in `src/adapters/driven/storage/db.rs` via CREATE TABLE statements in the migration function. Tables:

**Game-scoped:** `games`, `game_state_snapshots`, `messages`, `message_swipes`

**Global:** `llm_messages`, `prompt_presets`

**Seeded game data:** `worlds`, `maps`, `personas`, `characters`, `settings`

No column listings here — see the source for definitions.

## Seeding Pattern

Two functions called from `bootstrap::run()` at startup:

1. `ensure_presets(db_pool, data_dir)` — seeds prompt presets from `data/prompt_presets/`
2. `seed_game_data(storage, data_dir)` — seeds worlds, personas, characters from JSON files

**Contracts:**

- Idempotent — skips if key already exists (INSERT OR IGNORE or existence check)
- JSON files are seed templates only — DB is the sole source of truth at runtime
- Startup-blocking — seeding must complete before server starts
- `seed_world()` returns `world_id` for FK use by persona/character seeding
- Corrupt world.json files are skipped with a warning, not fatal

**Data flow:** `data/worlds/<key>/world.json` → `WorldManifest` → `WorldCard` (via `From<WorldManifest>`) → `Storage::seed_world()` → DB row. Personas and characters follow the same pattern.

## Settings Persistence

- DB-backed singleton (row with `id=1`)
- `AppSettings::save(storage)` writes to `settings` table
- `load_settings(storage)` reads from DB, falls back to defaults
- UI handlers persist changes via DB write-through

## Cross-Table Coordination

Storage methods touch exactly one table. Multi-table operations are free functions in `src/application/context.rs`, not methods on Storage. Examples:

- `save_message_and_snapshot(ctx, state)` — saves snapshot then message
- `load_messages_with_swipes(storage)` — loads messages then hydrates swipes

**Atomicity:** Sequential SQLite statements on a single connection. Tiny crash window; acceptable for non-critical data.

## GameStateSnapshot

Serializable subset of `GameState` for persistence. Messages excluded; hydrated separately. Lives in `crate::domain::model::state::game_state_snapshot` (domain-owned DTO; storage layer serializes/deserializes it to the `game_state_snapshots` table, but the type itself lives in the domain so the application layer does not import from `adapters::driven::storage` to reference a snapshot value).

## Testing Strategy

- InMemory backend for fast unit tests (no SQLite I/O)
- `with_test_failures()` returns `(Storage, TestFailureHandle)` for failure injection
- TestFailureHandle takes `&'static str` method-name keys + `TestOverride` failure payloads
- Test-infra types (`TestOverride`, `TestFailureHandle`) live in `src/adapters/driven/storage/backend/test_support.rs`, re-exported via `crate::adapters::driven::storage::backend::{TestFailureHandle, TestOverride}`
- See `src/adapters/driven/storage/backend/core.rs` for `Storage` constructors
