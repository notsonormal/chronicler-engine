# Specification: Engine Data Schemas

## Objective
Define the JSON data schemas used by the Chronicler Engine for characters, rooms, triggers, world definitions, and game state snapshots. Also documents the SQLite database schema for persistence.

## Storage Architecture

### JSON Seed Files
Game data is loaded from JSON seed files at startup and persisted to SQLite. After seeding, the database is the **sole source of truth** at runtime.
- `data/worlds/<key>/world.json` — World definitions
- `data/worlds/<key>/map.json` — Map definitions (co-located with world.json)
- `data/worlds/<key>/player.json` — Player persona
- `data/worlds/<key>/characters/*.json` — NPC definitions
- `data/personas/*.json` — Global personas
- `data/settings.json` — Application settings
- `data/prompt_presets/{system,quantifier}/*.json` — Prompt presets

### SQLite Database Schema (Migration v11)

#### Core Tables

**`games`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `world_name TEXT NOT NULL DEFAULT 'default'`
- `name TEXT NOT NULL DEFAULT 'Unnamed'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

**`game_state_snapshots`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE`
- `movement TEXT NOT NULL` (JSON)
- `narrative TEXT NOT NULL` (JSON)
- `scene TEXT NOT NULL` (JSON)
- `npc_encounter_log TEXT NOT NULL` (JSON)
- `created_at TEXT NOT NULL`
- Index: `idx_snapshots_game_latest (game_id, created_at DESC)`

**`messages`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE`
- `sender TEXT`
- `message_type TEXT NOT NULL`
- `timestamp TEXT NOT NULL`
- `active_swipe_index INTEGER NOT NULL DEFAULT 0`
- `is_deleted INTEGER NOT NULL DEFAULT 0`
- Index: `idx_messages_game_id (game_id, id)`

**`message_swipes`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE`
- `swipe_index INTEGER NOT NULL`
- `text TEXT NOT NULL`
- `snapshot_id INTEGER`
- `location_header TEXT`
- `event_header TEXT`
- Unique: `(message_id, swipe_index)`

**`llm_messages`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `agent_name TEXT NOT NULL`
- `backend_name TEXT NOT NULL`
- `model_name TEXT NOT NULL`
- `system_prompt TEXT NOT NULL`
- `user_prompt TEXT NOT NULL`
- `raw_request_json TEXT NOT NULL`
- `raw_response_json TEXT NOT NULL`
- `parsed_response TEXT NOT NULL`
- `error_message TEXT`
- `created_at TEXT NOT NULL`
- Index: `idx_llm_messages_created_at (created_at DESC)`

**`prompt_presets`**
- `id TEXT PRIMARY KEY`
- `name TEXT NOT NULL`
- `preset_type TEXT NOT NULL`
- `role TEXT`
- `instructions TEXT`
- `writing_style TEXT`
- `output_format TEXT`
- `is_default INTEGER NOT NULL DEFAULT 0`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- Index: `idx_prompt_presets_type (preset_type)`

#### Game Data Tables (Migration v11)

**World Seeding & Loading (Phase 3)**

On first startup (or if DB is empty), `bootstrap::ensure_defaults()` calls `seed_game_data()` which seeds worlds, personas, and characters from JSON files:
1. Scans `data/worlds/*/world.json` for all worlds
2. Deserializes `WorldManifest` (contains file pointers: `map_file`, `player_file`, `characters_dir`)
3. Converts to `WorldCard` via `From<WorldManifest>` (adds `key`, `player_key`, `default_scenario_id`)
4. Calls `Storage::seed_world(world_card, map)` — INSERT OR IGNORE (idempotent)
5. Loads `PlayerCard` from `data/personas/<player_file>` and calls `Storage::seed_persona(key, player)` — skip if exists
6. Loads `NpcCard`s from `data/characters/<characters_dir>/*.json` and seeds each — skip if exists

After seeding, runtime loading is 100% database-first:
- `Storage::get_world(key)` → `WorldCard + MapDef`
- `world_card.player_key` → `Storage::get_persona(key)` → `PlayerCard`
- `Storage::get_world_id(key)` → `list_characters(world_id)` → `Vec<NpcCard>`

**File I/O only during seeding**; runtime has zero filesystem coupling.

**`worlds`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `key TEXT NOT NULL UNIQUE` — Original string identifier (e.g., `redmist_estate`)
- `name TEXT NOT NULL`
- `description TEXT NOT NULL DEFAULT ''`
- `global_rules TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<String>`
- `starting_room_id TEXT NOT NULL DEFAULT 'start'`
- `scenarios TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<StartingScenario>`
- `default_scenario_id TEXT`
- `default_room_image TEXT`
- `player_key TEXT NOT NULL DEFAULT ''` — Filename stem of player persona (e.g., `julian` from `julian.json`). Determines which persona is the player character for this world. Falls back to `"player"` if empty.
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

**`maps`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `world_id INTEGER NOT NULL REFERENCES worlds(id) ON DELETE CASCADE`
- `map_data TEXT NOT NULL` — JSON: full `MapDef` (overworld, regions, rooms)
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- Index: `idx_maps_world (world_id)`

**`personas`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `key TEXT NOT NULL UNIQUE` — Filename stem (e.g., `julian`)
- `name TEXT NOT NULL`
- `description TEXT NOT NULL DEFAULT ''`
- `personality TEXT NOT NULL DEFAULT ''`
- `scenario TEXT NOT NULL DEFAULT ''`
- `example_dialogue TEXT NOT NULL DEFAULT ''`
- `summary TEXT`
- `profile_image TEXT`
- `headshot_image TEXT`
- `inventory TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<String>`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

**`characters`**
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `key TEXT NOT NULL` — From `NpcCard.id` (e.g., `elena_voss`)
- `world_id INTEGER NOT NULL REFERENCES worlds(id) ON DELETE CASCADE`
- `name TEXT NOT NULL`
- `description TEXT NOT NULL DEFAULT ''`
- `personality TEXT NOT NULL DEFAULT ''`
- `scenario TEXT NOT NULL DEFAULT ''`
- `example_dialogue TEXT NOT NULL DEFAULT ''`
- `summary TEXT`
- `profile_image TEXT`
- `headshot_image TEXT`
- `inventory TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<String>`
- `triggers TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<Trigger>`
- `relationships TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<Relationship>`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- Unique: `(key, world_id)`
- Index: `idx_characters_world (world_id)`

**`settings`**
- `id INTEGER PRIMARY KEY CHECK (id = 1)` — Singleton row
- `connections TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<Connection>`
- `narration_connection_id TEXT NOT NULL DEFAULT 'openrouter-gpt-4o-mini'`
- `quantifier_connection_id TEXT NOT NULL DEFAULT 'openrouter-gpt-4o-mini'`
- `response_length TEXT NOT NULL DEFAULT ''`
- `text_check TEXT NOT NULL DEFAULT '{}'` — JSON: `TextCheckSettings`
- `agents TEXT NOT NULL DEFAULT '[]'` — JSON: `Vec<AgentConfig>`
- `active_system_prompt_preset_id TEXT NOT NULL DEFAULT 'system_default'`
- `active_quantifier_prompt_preset_id TEXT NOT NULL DEFAULT 'quantifier_default'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

## Data Normalization Rules
Character JSON files contained in `data/characters/` should be scrubbed using regular expressions or textual parsing to locate:
- `Personality: [text]` -> moved to `NpcCard.personality`
- `Background: [text]` / `Goals: [text]` -> moved to `NpcCard.scenario`
- `Appearance: [text]` -> Extract and leave inside `NpcCard.description` alongside the general introduction text.

## Redmist Estate Map Topology
The layout for the initial demo map features an Overworld with a single region (Redmist Mansion) comprising 5 distinct rooms:

1. **Front Gates**: Connected North to `Entrance Hall`. Contains: `carla`.
2. **Entrance Hall**: Connected South to `Front Gates`, West to `Kitchen`, East to `Financial Office`, North to `Master Quarters`. Contains: `gabriella`.
3. **Kitchen**: Connected East to `Entrance Hall`. Contains: `louise`.
4. **Financial Office**: Connected West to `Entrance Hall`. Contains: `jezebel`.
5. **Master Quarters**: Connected South to `Entrance Hall`. Contains: `lisette`.

## Implementation Requirements
- Create `data/world/map.json`.
- Modify `main.rs` to load the `.json` files from disk upon game boot (using `std::fs::read_to_string` and `serde_json`), deprecating the hardcoded "Aethelgard" mock data.

## Character Schema (PlayerCard and NpcCard)
Both `PlayerCard` and `NpcCard` share this unified structure for narrative fields:

```json
{
  "name": "string",
  "description": "string (physical appearance + general intro)",
  "personality": "string (e.g., 'Arrogant, brave, tech-savvy')",
  "scenario": "string (background or current motivation)",
  "example_dialogue": "string (optional example for LLM context)",
  "inventory": ["item_id_1", "item_id_2"],  // Only on PlayerCard/NpcCard, NOT CharacterSheet
  "profile_image": "string (optional, preferred profile image)",
  "summary": "string (optional, brief character summary)",
  "headshot_image": "string (optional, headshot/portrait for sidebar grid)"
}
```

### Image Field Usage
- `profile_image`: Preferred for character profile display
- `headshot_image`: Used for visual sidebar NPC portraits (2-column grid)
  - Falls back to `profile_image` if not set

This schema allows the LLM Game Master to treat player and NPCs with equal granular detail.

## Room Schema (Current)
Rooms in map.json have the following structure:

```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "exits": { "north": "room_id", "east": "room_id" },  // Cardinal direction exits
  "items": ["item_id_1"],
  "navigation_description": "string (optional, custom movement narration)",
  "image_path": "string"
}
```

## NpcCard Schema (Current)
```json
{
  "id": "string",
  "name": "string",
  "description": "string (physical appearance + general intro)",
  "personality": "string (e.g., 'Arrogant, brave, tech-savvy')",
  "scenario": "string (background or current motivation)",
  "example_dialogue": "string (optional example for LLM context)",
  "inventory": ["item_id_1", "item_id_2"],
  "profile_image": "string (optional, preferred profile image)",
  "headshot_image": "string (optional, headshot/portrait for sidebar grid)",
  "triggers": [Trigger, ...],  // Array of Trigger objects. Defaults to [] if missing.
  "relationships": [
    {
      "with": "npc_id",
      "dynamic": "current relationship state (e.g. 'tense rivalry')",
      "static": "underlying relationship fact (e.g. 'They are sisters')"
    }
  ]  // Defaults to [] if missing.
}
```

## Trigger Schema (NEW)
Attached to an NPC. Defines a condition and the narration to inject when that condition is met.

```json
{
  "condition": { "TimesMet": ["Eq", 0] },
  "action": { "narration_prompt": "The shopkeeper looks up from behind the counter with a warm smile." },
  "repeat": false
}
```

### Fields
- `condition`: The condition that must be true for this trigger to fire. Currently supports `TimesMet` with a comparison operator.
  - `TimesMet`: Array of `[operator, value]`. Operators: `Eq` (equal), `Lt` (less than), `Gte` (greater than or equal)
- `action.narration_prompt`: The text injected into the continuation LLM prompt when this trigger fires
- `repeat`: If `false`, fires only once (first time condition is met). If `true`, fires whenever condition is met.
- `room_id` (optional): If set, this trigger only fires when the player is in this room. If omitted or `null`, the trigger is global.

## NpcEncounterState Schema (NEW)
Tracks character state for a specific NPC. Stored in `GameState.npc_encounter_log`.

```json
{
  "times_met": 0,
  "trigger_fired": {},
  "currently_meeting": false
}
```

### Fields
- `times_met`: How many times the player has encountered this NPC
- `trigger_fired`: Map of trigger index (usize) to boolean (whether that trigger has fired)
- `currently_meeting`: Whether the player is currently in the same room/session as this NPC

## NpcEncounterLog Schema (NEW)
Contains all NPC encounter state. Top-level field in `GameState`.

```json
{
  "npcs": {}
}
```

### Fields
- `npcs`: Map of NPC ID to `NpcEncounterState`

## NpcEventType Schema (NEW)
Enum representing NPC movement event types.

```json
"Entered" | "Left"
```

### Variants
- `Entered`: NPC transitioned from not being in the area to being in the area
- `Left`: NPC transitioned from being in the area to not being in the area

## NpcEvent Schema (NEW)
A single NPC movement event.

```json
{
  "npc_id": "carla",
  "event_type": "Entered"
}
```

## NpcEventList Schema (NEW)
Collection of NPC movement events with confidence level. Returned by `compute_npc_events()`.

```json
{
  "events": [
    { "npc_id": "carla", "event_type": "Entered" },
    { "npc_id": "derek", "event_type": "Left" }
  ],
  "confidence": "Medium"
}
```

### Fields
- `events`: Array of `NpcEvent` objects
- `confidence`: Confidence level (`High`, `Medium`, `Low`). Medium when events detected, Low when no events.

## Swipe Schema (NEW)
A single alternative generation for a message. Stored in the `message_swipes` table.

```json
{
  "text": "string",
  "snapshot_id": 42,
  "location_header": "string (optional)",
  "event_header": "string (optional)"
}
```

### Fields
- `text`: The generated narrative text for this swipe
- `snapshot_id`: Reference to the `GameStateSnapshot` that produced this text. Nullable for the initial swipe (no snapshot yet).
- `location_header`: Location header associated with this swipe (if any)
- `event_header`: Event header associated with this swipe (if any)

## Message Schema (Updated)
Core narrative unit with swipe support.

```json
{
  "id": 1,
  "sender": "Game Master",
  "text": "You enter the hall.",
  "message_type": "Narration",
  "timestamp": "2026-05-24T12:00:00Z",
  "location_header": "Entrance Hall",
  "event_header": null,
  "snapshot_id": 42,
  "active_swipe_index": 0,
  "is_deleted": false,
  "swipes": [
    {
      "text": "You enter the hall.",
      "snapshot_id": 42,
      "location_header": "Entrance Hall",
      "event_header": null
    }
  ]
}
```

### Fields
- `id`: Auto-incrementing message ID
- `sender`: Optional sender name (None for narration, "Player" for input)
- `text`: Active swipe text (hydrated from `swipes[active_swipe_index]`)
- `message_type`: `Narration`, `Dialogue`, `System`, or `Input`
- `timestamp`: UTC timestamp
- `location_header`: Active location header (from active swipe)
- `event_header`: Active event header (from active swipe)
- `snapshot_id`: Active snapshot ID (from active swipe)
- `active_swipe_index`: Index of the currently displayed swipe
- `is_deleted`: Soft-delete flag (true for messages temporarily hidden during retry)
- `swipes`: Array of all swipes for this message

## WorldCard Schema (NEW)
Top-level world definition loaded from `data/worlds/*/world.json`.

```json
{
  "name": "string",
  "description": "string",
  "global_rules": ["rule 1", "rule 2"],
  "starting_room_id": "string",
  "scenarios": [
    {
      "id": "string",
      "name": "string",
      "description": "string",
      "starting_room_id": "string",
      "text": "string (optional)",
      "npcs": ["npc_id_1"]
    }
  ],
  "default_room_image": "string (optional, fallback image for rooms without one)"
}
```

### Fields
- `name`: Display name of the world
- `description`: Lore and setting description for the Game Master
- `global_rules`: Array of global behavioral rules injected into the system prompt
- `default_room_image`: Optional fallback image path used when a room does not specify its own `image_path`
