# Specification: Redmist Estate Map and Data Parsing

## Objective
Normalize legacy character cards (which bundle multiple disparate fields like Personality and Background into the standard `description` field) to match the separated field model established in the Chronicler Engine `NpcCard`. 
Create a global dynamic map to place these characters inside the engine.

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
  "inventory": ["item_id_1", "item_id_2"],
  "image_path": "string (optional, legacy field - full body image)",
  "profile_image": "string (optional, preferred profile image)",
  "headshot_image": "string (optional, headshot/portrait for sidebar grid)"
}
```

### Image Field Usage
- `image_path`: Legacy field, full body image
- `profile_image`: Preferred for character profile display
- `headshot_image`: Used for visual sidebar NPC portraits (2-column grid)
  - Falls back to `image_path` if not set

This schema allows the LLM Game Master to treat player and NPCs with equal granular detail.

## Room Schema (Current)
Rooms in map.json have the following structure:

```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "exits": { "north": "room_id", "east": "room_id" },  // Legacy cardinal directions
  "semantic_exits": [  // NEW: Semantic triggers for natural language
    {
      "trigger": "front gate",
      "destination": "entrance_hall",
      "keywords": ["enter", "go through", "pass through"]
    }
  ],
  "items": ["item_id_1"],
  "npcs": ["npc_id_1"],
  "image_path": "string"
}
```

### Semantic Exits
The `semantic_exits` array enables quantifier-driven movement:
- `trigger`: The text that activates this exit (e.g., "front gate")
- `destination`: The room ID to navigate to
- `keywords`: Additional matching patterns for flexible input (e.g., "go through", "enter")

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
  "triggers": [Trigger, ...]  // NEW: Array of Trigger objects. Defaults to [] if missing.
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
Tracks character state for a specific NPC. Stored in `GameState.character_state`.

```json
{
  "times_met": 0,
  "trigger_fired": {}
}
```

### Fields
- `times_met`: How many times the player has encountered this NPC
- `trigger_fired`: Map of trigger description to boolean (whether that trigger has fired)

## CharacterState Schema (NEW)
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
