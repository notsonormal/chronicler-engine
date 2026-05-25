# Specification: Engine Data Schemas

## Objective
Define the JSON data schemas used by the Chronicler Engine for characters, rooms, triggers, world definitions, and game state snapshots.

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

## CharacterSheet Schema (Current)
A unified structure for both `PlayerCard` and `NpcCard` narrative fields:

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
  "log_type": "Narration",
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
- `log_type`: `Narration`, `Dialogue`, `System`, or `Input`
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
