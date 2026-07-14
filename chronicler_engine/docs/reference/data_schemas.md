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
   - Scans `data/personas/*.json` and seeds each as a `PersonaCard` via `Storage::seed_persona(key, player)` — idempotent
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
  "key": "string (filename stem; primary identifier when seeded)",
  "name": "string",
  "description": "string (physical appearance + general intro)",
  "personality": "string (e.g., 'Arrogant, brave, tech-savvy')",
  "scenario": "string (background or current motivation)",
  "example_dialogue": "string (optional example for LLM context)",
  "inventory": ["item_id_1", "item_id_2"],
  "profile_image": "string (optional, preferred profile image)",
  "summary": "string (optional, brief character summary)",
  "headshot_image": "string (optional, headshot/portrait for sidebar grid)"
}
```

The `key` field is the filename stem for the persona/character at seed time (e.g. `data/personas/julian.json` → `key = "julian"`); for NpcCard it is derived from the JSON filename within `data/characters/<characters_dir>/`. Both fields are required on the live struct.

### Image Field Usage

- `profile_image`: Preferred for character profile display
- `headshot_image`: Used for visual sidebar NPC portraits (2-column grid)
  - Falls back to `profile_image` if not set

This schema allows the LLM Game Master to treat player and NPCs with equal granular detail.

## Room Schema

Rooms in `map.json` define per-tile narrative state with cardinal exits. Exits use the lowercase-serialized `Direction` enum (`north`, `south`, `east`, `west`, `up`, `down`).

```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "exits": { "north": "room_id", "east": "room_id" },
  "items": ["item_id_1"],
  "navigation_description": "string (optional, custom movement narration)",
  "image_path": "string (optional, nullable)"
}
```

`image_path` is `Option<String>` with `#[serde(default)]`; omit or use `null` when no image is available.

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

Attached to an NPC. Defines a requirement and the narration to inject when that requirement is met. The live `Trigger` struct has the shape `{ requirement, narration, repeat, room_id? }`:

```json
{
  "requirement": { "TimesMet": { "operator": "Eq", "threshold": 0 } },
  "narration": {
    "name": "FirstMeeting",
    "narration_prompt": "The shopkeeper looks up from behind the counter with a warm smile."
  },
  "repeat": false,
  "room_id": null
}
```

### Fields

- `requirement`: The requirement that must hold for the trigger to fire. The `TimesMet` variant takes `operator` + `threshold`. Operators: `Eq` (equal), `Lt` (less than), `Gte` (greater than or equal).
- `narration.name`: Display name for the trigger (used in logs and the event header).
- `narration.narration_prompt`: The text injected into the continuation LLM prompt when this trigger fires.
- `repeat`: If `false`, fires only once (first time condition is met). If `true`, fires whenever condition is met.
- `room_id` (optional): If set, this trigger only fires when the player is in this room. If omitted or `null`, the trigger is global.

## NPC Event Schemas

`compute_npc_events()` diffs the previous vs current NPC sets and returns an `NpcEventList` with the per-NPC transition type and an aggregate confidence level:

```json
{
  "events": [
    { "npc_id": "carla", "event_type": "Entered" },
    { "npc_id": "derek", "event_type": "Left" }
  ],
  "confidence": "Medium"
}
```

- `events`: Array of `NpcEvent { npc_id, event_type }`. `event_type` is `Entered` (was absent, now present) or `Left` (was present, now absent).
- `confidence`: `High` (≥1 event with high confidence), `Medium` (events detected at medium confidence), `Low` (no events).

Per-NPC encounter state is held in `GameState.npc_encounter_log`, mapping each NPC id to `{ times_met, trigger_fired, currently_meeting }`. `times_met` increments on `Entered` transitions only; `currently_meeting` mirrors current presence.

## Swipe + Message Accessor Pattern

`Message` holds direct fields (`id`, `sender`, `message_type`, `timestamp`, `active_swipe_index`, `is_deleted`, `swipes`). The narrative content lives on the active swipe and is exposed via accessor methods only — never as direct top-level fields on `Message`. Accessor definitions live in `src/domain/model/message.rs`.

A `Swipe` is `{ text, snapshot_id, location_header, event_header }`. `snapshot_id` is `None` for the initial swipe (no snapshot existed before its creation); for subsequent swipes it references the `GameStateSnapshot` that produced the text, enabling state-consistent switching.

`sender` is `Option<String>` (`None` for narration; `"Player"` for input). `message_type` is one of `Narration`, `Dialogue`, `System`, `Input`.

## WorldManifest Schema

Top-level world definition loaded from `data/worlds/*/world.json`. This is the on-disk file shape; the runtime `WorldCard` is derived from it via `From<WorldManifest>` (which strips `map_file` and `characters_dir` and resolves `default_scenario_id`).

```json
{
  "id": "string (primary identifier; becomes world_key on disk)",
  "name": "string",
  "description": "string",
  "global_rules": ["rule 1", "rule 2"],
  "map_file": "string (relative path to map.json within the world folder)",
  "characters_dir": "string (subdirectory under world folder containing NPC JSON files)",
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
  "default_scenario_id": "string (id of the default scenario; resolved at load time)",
  "default_room_image": "string (optional, fallback image for rooms without one)"
}
```

### Fields

- `id`: World identifier. Becomes `world_key` when persisted; readers use `Storage::get_world(key)`.
- `name`: Display name of the world.
- `description`: Lore and setting description for the Game Master.
- `global_rules`: Array of global behavioral rules injected into the system prompt.
- `map_file` / `characters_dir`: File pointers used by the seeding flow; stripped when converting to `WorldCard`.
- `scenarios[].starting_room_id`: Default room id for the scenario (serde default `"start"`).
- `default_scenario_id`: Resolved at load time to determine `WorldCard::starting_room_id()`.

## Document References

- [ADR-026: Relocate Persona Binding from World to Game](../adr/adr-026-persona-relocation-to-game.md) — persona binding relocation that informed idempotent seeding
- [system/message_model.md](../system/message_model.md) — accessor pattern: `text()` + `location_header()` + `event_header()` + `snapshot_id()` read from active swipe
