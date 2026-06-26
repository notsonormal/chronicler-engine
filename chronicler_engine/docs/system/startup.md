# Engine Startup & Initialization

This document defines the authoritative sequence for bootstrapping the Chronicler Engine, from environment resolution to world state creation.

## 1. Data Path Resolution

The engine resolves its `data/` directory using the following priority:

1. **Executable Proximity**: `data/` folder in the same directory as the engine binary (portable deployments).
2. **Current Working Directory**: `./data` (standard development mode).

## 2. Two-Phase Bootstrap: Seed, Then DB-First

Startup operates in two phases with a strict boundary:

**Phase 1 — File Seeding (once, at boot):** JSON files under `data/` are read and inserted into SQLite. This is idempotent — existing rows are skipped. After seeding, the files are never read again.

**Phase 2 — DB-First (all runtime):** Every subsequent read comes from the database. `WorldCard` has no knowledge of filesystem paths; `WorldManifest` exists only for parsing seed files.

This boundary is load-bearing: if runtime code reaches for JSON files instead of the DB, it breaks the invariant that the DB is the sole source of truth.

### Seeding Order

Seeding proceeds in dependency order to satisfy foreign keys:

1. Worlds and maps
2. Personas (referencing world)
3. Characters (referencing world)
4. Prompt presets (independent)

Corrupt seed files are skipped with a warning — seeding is not fatal.

### Game State Initialization

`build_fresh_initial_state` resolves the active world's default scenario (via `WorldCard::default_scenario()`) and reads `scenario.starting_room_id` to set the player's initial room. If no scenarios exist, it falls back to `"start"`.

Settings are loaded from the database once during bootstrap, wrapped in `Arc<RwLock<AppSettings>>`, and passed through the construction chain. No business logic layer reloads settings from disk. Connection changes require a server restart. Only `max_context_tokens` is read dynamically at runtime.

## 4. Server Startup

The Axum HTTP server starts after game state synthesis. It binds to the configured `--port` and serves static files at `/assets` and `/data` routes, with `/assets` as the fallback for unmatched routes.
