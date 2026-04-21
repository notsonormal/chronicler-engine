# Engine Startup & Initialization

This document defines the authoritative sequence for bootstrapping the Chronicler Engine, from environment resolution to world state creation.

## 1. Data Path Resolution
The engine resolves its `data/` directory using the following priority:
1.  **Environment Variable**: `CHRONICLER_DATA` (overrides all others).
2.  **Executable Proximity**: Checks for a `data/` folder in the same directory as the engine binary (for portable deployments).
3.  **Current Working Directory**: Defaults to `./data` (standard development mode).

## 2. World Initialization Sequence
When a world is loaded via the `--world` flag, the engine performs these steps:
1.  **Manifest Loading**: Reads `world.json` to identify the map, player, and starting room files.
2.  **Map Deserialization**: Loads the `MapDef` (Regions, Rooms, Exits).
3.  **Player Bootstrapping**: Loads the `PlayerCard` (Sheet and Inventory).
4.  **NPC Discovery**: Scans the `characters/` directory within the world folder, deserializing every `.json` file into an `NpcCard`.
5.  **State Synthesis**: Combines these components into a unified `GameState`, initializing the current location and visibility filters.

## 3. Server Startup
The HTTP server (Axum) is initialized after the game state is synthesized. It binds to the specified `--port` and mounts the `assets/` and `data/` directories for static resource serving.
