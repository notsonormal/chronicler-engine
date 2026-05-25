# Engine Startup & Initialization



This document defines the authoritative sequence for bootstrapping the Chronicler Engine, from environment resolution to world state creation.

## 1. Data Path Resolution
The engine resolves its `data/` directory using the following priority:
1.  **Executable Proximity**: Checks for a `data/` folder in the same directory as the engine binary (for portable deployments).
2.  **Current Working Directory**: Defaults to `./data` (standard development mode).

## 2. World Initialization Sequence
When a world is loaded via the `--world` flag, the engine performs these steps:
1.  **Manifest Loading**: Reads `world.json` to identify the map, player, and starting room files.
2.  **Map Deserialization**: Loads the `MapDef` (Regions, Rooms, Exits).
3.  **Player Bootstrapping**: Loads the `PlayerCard` (Sheet and Inventory).
4.  **NPC Discovery**: Scans the `data/characters/<characters_dir>/` directory (where `characters_dir` comes from `world.json`, defaulting to the world id), deserializing every `.json` file into an `NpcCard`.
5.  **State Synthesis**: Combines these components into a unified `GameState`, initializing the current location and visibility filters.

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
The HTTP server (Axum) is initialized after the game state is synthesized. It binds to the specified `--port` and mounts the `assets/` and `data/` directories for static resource serving.
