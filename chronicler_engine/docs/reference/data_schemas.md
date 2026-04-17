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
- `headshot_image**: Used for visual sidebar NPC portraits (2-column grid)
  - Falls back to `image_path` if not set

This schema allows the LLM Game Master to treat player and NPCs with equal granular detail.
