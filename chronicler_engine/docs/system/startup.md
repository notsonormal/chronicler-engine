# Engine Startup & Initialization



This document defines the authoritative sequence for bootstrapping the Chronicler Engine, from environment resolution to world state creation.

## 1. Data Path Resolution
The engine resolves its `data/` directory using the following priority:
1.  **Executable Proximity**: Checks for a `data/` folder in the same directory as the engine binary (for portable deployments).
2.  **Current Working Directory**: Defaults to `./data` (standard development mode).

## 2. World Initialization Sequence (Phases 1-2: File Seeding)
On first startup (or if the DB is empty), JSON files are seeded into the SQLite database:
1.  **World Directory Scan**: Scans `data/worlds/*/world.json` for all available worlds
2.  **Manifest Processing**: For each `world.json`:
    - Deserializes `WorldManifest` (contains file pointers: `map_file`, `player_file`, `characters_dir`)
    - Converts to `WorldCard` via `From<WorldManifest>` (adds `key`, `player_key`, `default_scenario_id`)
    - Loads `MapDef` from `data/worlds/<id>/<map_file>`
    - Calls `Storage::seed_world(world_card, map)` — idempotent INSERT OR IGNORE
    - Loads `PlayerCard` from `data/personas/<player_file>` and calls `Storage::seed_persona(key, player)` — skip if exists
    - Loads `NpcCard`s from `data/characters/<characters_dir>/*.json` and calls `Storage::seed_character(world_id, npc)` for each — skip if exists
3.  **Preset Seeding**: Prompt presets from `data/prompt_presets/` are also seeded idempotently

## 3. Runtime World Loading (Phase 3: DB-First)
After seeding, all runtime data comes from the database:
1.  **DB Query**: `Storage::get_world(world_key)` returns `WorldCard + MapDef` from SQLite
2.  **Persona Loading**: `Storage::get_persona(world_card.player_key)` returns `PlayerCard`
3.  **Character Loading**: `Storage::list_characters(world_id)` returns `Vec<NpcCard>`
4.  **State Synthesis**: Components combined into `GameState` with injection of scenario logs

**File pointers removed**: After seeding, `WorldCard` has no knowledge of filesystem paths. The `WorldManifest` type exists only for parsing seed files.

## 3. Binary Bootstrap (`main.rs` → `bootstrap.rs` + `cli.rs`)
The binary entry point (`src/main.rs`) delegates to two focused modules:
- **`cli.rs`**: Parses command-line arguments via `clap` (`Args` struct with `--world`, `--port`, `--list-worlds`).
- **`bootstrap.rs`**: Orchestrates the full startup sequence — resolves the data directory, loads the selected world, initializes game state, constructs the `GameService`, and starts the Axum HTTP server.

This split keeps `main.rs` minimal and makes the bootstrap logic independently testable.

## 4. Settings Loading
Settings are loaded **once** during bootstrap:
1. **`bootstrap/run.rs`** calls `load_settings()` and wraps the result in `Arc<RwLock<AppSettings>>`
2. The `Arc<RwLock<AppSettings>>` is passed to `run_server_with_config()`
3. `AppState` stores it; `GameServiceContext` carries it; `DefaultGameService` receives it at construction time
4. Backends (`OpenRouterBackend`, `OllamaBackend`) store a clone of the `Arc` for settings access

No business logic layer reloads settings from disk. Connection changes still require a server restart.

## 5. Server Startup
The HTTP server (Axum) is initialized after the game state is synthesized. It binds to the specified `--port` and serves static files: `/assets` and `/data` routes via `tower_http::services::ServeDir`, with `/assets` as the fallback for unmatched routes.
