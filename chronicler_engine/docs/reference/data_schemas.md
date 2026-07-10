# Specification: Engine Data Schemas

## Objective

Define the JSON data schemas used by the Chronicler Engine for characters, rooms, triggers, world definitions, and game state snapshots. Also documents the SQLite database schema for persistence.

## Storage Architecture

### JSON Seed Files

Game data is loaded from JSON seed files at startup and persisted to SQLite. After seeding, the database is the **sole source of truth** at runtime.

- `data/worlds/<key>/world.json` — World definitions
- `data/worlds/<key>/map.json` — Map definitions (co-located with world.json)
- `data/worlds/<key>/characters/*.json` — NPC definitions
- `data/personas/*.json` — Global personas
- `data/settings.json` — Application settings
- `data/prompt_presets/{system,quantifier}/*.json` — Prompt presets

### SQLite Database Schema

Tables for runtime persistence. The full column-level DDL lives in `src/adapters/driven/storage/db.rs`; column-by-column listings are intentionally not duplicated here.

Core tables: `games`, `game_state_snapshots`, `messages`, `message_swipes`, `llm_messages`, `prompt_presets`.

Game data tables: `worlds`, `maps`, `personas`, `characters`, `settings`.

#### World Seeding & Loading

On first startup (or if DB is empty), `bootstrap::run()` calls seeding functions:

1. **Prompt Presets:** `ensure_presets()` seeds system and quantifier prompt presets
2. **Game Data:** `seed_game_data()` seeds worlds, personas, and characters from JSON files:
   - Scans `data/worlds/*/world.json` for all worlds
   - Deserializes `WorldManifest` (file pointers: `map_file`, `characters_dir`)
   - Converts to `WorldCard` via `From<WorldManifest>` (adds `key`, `default_scenario_id`)
   - Calls `Storage::seed_world(world_card, map)` → returns `world_id: i64` (idempotent)
   - Scans `data/personas/*.json` and seeds each as a `PersonaCard` via `Storage::seed_persona(key, player)` — idempotent (see ADR-026)
   - Loads `NpcCard`s from `data/characters/<characters_dir>/*.json` and seeds each via `seed_character(world_id, npc)` — skip if exists

After seeding, runtime loading is 100% database-first:

- `Storage::get_world(key)` → `WorldWithMap { world_id, world_card, map }` (uses `DbWorld::from_row()`)
- `game.persona_key` → `Storage::get_persona(key)` → `PersonaCard` (the persona is bound on the game row, not the world)
- `world_with_map.world_id` → `Storage::list_characters(world_id)` → `Vec<NpcCard>`

**File I/O only during seeding**; runtime reads only from the database.

**Pattern Consistency**: World storage uses `DbWorld::from_row()` + `world_card_from_db()` conversion function, matching persona and character storage.

## Character Schema (PersonaCard and NpcCard)

Both `PersonaCard` and `NpcCard` share this unified structure for narrative fields:

```json
{
  "name": "string",
  "description": "string (physical appearance + general intro)",
  "personality": "string (e.g., 'Arrogant, brave, tech-savvy')",
  "scenario": "string (background or current motivation)",
  "example_dialogue": "string (optional example for LLM context)",
  "inventory": ["item_id_1", "item_id_2"],  // Only on PersonaCard/NpcCard, NOT CharacterSheet
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

## Room Schema

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

## NpcCard Schema

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

## Trigger Schema

Attached to an NPC. Defines a condition and the narration to inject when that condition is met.

```json
{
  "name": "FirstMeeting",
  "condition": { "TimesMet": ["Eq", 0] },
  "action": { "narration_prompt": "The shopkeeper looks up from behind the counter with a warm smile." },
  "repeat": false
}
```

### Fields

- `name`: Display name for the trigger (used in logs and the `trigger_fired` map keys).
- `condition`: The condition that must be true for this trigger to fire. Currently supports `TimesMet` with a comparison operator.
  - `TimesMet`: Array of `[operator, value]`. Operators: `Eq` (equal), `Lt` (less than), `Gte` (greater than or equal)
- `action.narration_prompt`: The text injected into the continuation LLM prompt when this trigger fires
- `repeat`: If `false`, fires only once (first time condition is met). If `true`, fires whenever condition is met.
- `room_id` (optional): If set, this trigger only fires when the player is in this room. If omitted or `null`, the trigger is global.

## NpcEncounterState Schema

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

## NpcEncounterLog Schema

Contains all NPC encounter state. Top-level field in `GameState`.

```json
{
  "npcs": {}
}
```

### Fields

- `npcs`: Map of NPC ID to `NpcEncounterState`

## NpcEventType Schema

Enum representing NPC movement event types.

```json
"Entered" | "Left"
```

### Variants

- `Entered`: NPC transitioned from not being in the area to being in the area
- `Left`: NPC transitioned from being in the area to not being in the area

## NpcEvent Schema

A single NPC movement event.

```json
{
  "npc_id": "carla",
  "event_type": "Entered"
}
```

## NpcEventList Schema

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

## Swipe Schema

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

## Message Schema

Core narrative unit with swipe support. The persisted struct holds direct fields only; swipe-derived values are exposed via accessor methods. See [`system/message_model.md`](../system/message_model.md) for the accessor pattern.

```json
{
  "id": 1,
  "sender": "Game Master",
  "message_type": "Narration",
  "timestamp": "2026-05-24T12:00:00Z",
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

### Direct Fields

- `id`: Auto-incrementing message ID
- `sender`: Optional sender name (None for narration, "Player" for input)
- `message_type`: `Narration`, `Dialogue`, `System`, or `Input`
- `timestamp`: UTC timestamp
- `active_swipe_index`: Index of the currently displayed swipe
- `is_deleted`: Soft-delete flag (true for messages temporarily hidden during retry)
- `swipes`: Array of all swipes for this message

### Accessor Methods (NOT direct fields)

Swipe-derived values (`text`, `location_header`, `event_header`, `snapshot_id`) are exposed via accessor methods that read from the active swipe — they are not persisted as direct top-level fields. See [`system/message_model.md`](../system/message_model.md) for the accessor pattern and rationale.

## WorldCard Schema

Top-level world definition loaded from `data/worlds/*/world.json`.

```json
{
  "name": "string",
  "description": "string",
  "global_rules": ["rule 1", "rule 2"],
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
- `scenarios[].starting_room_id`: Default room ID for a given starting scenario (serde default `"start"`)
