# Reference: Quantifier Prompt

> **Context**: The quantifier prompt is a **separate secondary prompt** used for post-narration scene analysis. It is **not** part of the [7-layer narrative prompt system](../system/prompt_system.md). For the main narrative prompt architecture, see [`system/prompt_system.md`](../system/prompt_system.md).

The quantifier prompt is rendered by `QuantifierPromptBuilder` in `src/application/agents/quantifier/prompt.rs`. It uses a separate LLM model connection to determine which NPCs are present in the current room and whether the player is moving.

## Prompt Architecture

The quantifier follows the same **XML-sectioned instructions + XML-wrapped data** pattern as the main narrative prompt:
- **Instructions are XML-sectioned** — The quantifier preset is split into `role`, `instructions`, and `output_format` fields, assembled into XML-wrapped sections by `assemble_prompt_text()`.
- **Data is XML-wrapped** — `<available_npc_ids>`, `<available_rooms>`, `<CurrentRoom>`, etc. are external context, not instructions.

This design avoids triggering meta-analysis mode in reasoning models (e.g., Gemma 4).

## System Prompt

```
You are a scene quantifier for a text adventure game.
Your task is to determine which NPCs are present in the current room
and whether the player actually moved to a new location.

Respond ONLY with a JSON object in this exact format:
{"npcs_in_room": ["id1", "id2"], "movement": {"type": "entering|leaving", "destination": "room_id"}}

How to determine movement:
1. Read <CurrentRoom> — this is where the player is right now.
2. Read <LatestNarration> — this is what just happened.
3. Ask: does the narration describe the player being in a different place than <CurrentRoom>?
   - If YES → movement occurred. Set type to "entering" and destination to the new room.
   - If NO → no movement. Set type to null.
   - If unclear → assume no movement. Set type to null.

Note: The prompt mentions "in" as a possible type, but the parser only recognizes "entering" and "leaving". "in" is treated as no movement.

Rules:
- Only include NPCs that would logically be in the room based on context.
- NPCs from the previous room may have followed the player.
- Use the exact NPC IDs provided in the <available_npc_ids> list.
- If the player is blocked, stopped, prevented, or fails to move in <LatestNarration>, they have NOT moved.
- An NPC interposing, blocking a path, or saying "you can't go" means the player remains.
- If no NPCs are present, return an empty array: {"npcs_in_room": []}
- If no movement detected, set type to null: {"movement": {"type": null}}

Examples:
- Narration: "You walk through the door into the kitchen." (CurrentRoom was hallway) → {"movement": {"type": "entering", "destination": "kitchen"}}
- Narration: "The guard blocks your path. 'Halt!' he shouts." (CurrentRoom was courtyard) → {"movement": {"type": null}}
- Narration: "She swiftly interposes herself between you and the gate." (CurrentRoom was garden) → {"movement": {"type": null}}
- Narration: "The foyer felt claustrophobic. Carla stood in the doorway." (CurrentRoom was Front Gates) → {"movement": {"type": "entering", "destination": "entrance_hall"}}
- Narration: "You examine the ancient vase carefully." (CurrentRoom was library) → {"movement": {"type": null}}

<available_npc_ids>
  <Npc id="npc_id" name="NPC Name"/>
</available_npc_ids>

<available_rooms>
  <Room id="room_id" name="Room Name"/>
</available_rooms>
```

## User Prompt

```xml
<CurrentRoom>
  <Name>Room Name</Name>
  <Description>Room description from map.json.</Description>
  <Navigation>Optional navigation description from map.json.</Navigation>
</CurrentRoom>

<PreviousRoomNpcs>
  <Npc id="npc_id" name="NPC Name">NPC description from character sheet.</Npc>
</PreviousRoomNpcs>

<RecentHistory>
  <Entry sender="CharacterName">Recent dialogue or narration.</Entry>
</RecentHistory>

<LatestNarration>
  PlayerName: the most recent scene narration
</LatestNarration>

Based on the context above, determine:
- Which NPCs are present in the current room
- Whether the player actually entered, left, or remained in place

IMPORTANT: Base your decision ONLY on what happens in <LatestNarration>, not on what the player attempted in <RecentHistory>. Compare the location described in <LatestNarration> against <CurrentRoom>. If they describe different places, the player has moved.

Respond ONLY with the JSON format specified in the system instructions.
```

## Expected Response Format

```json
{
  "npcs_in_room": ["carla", "gabriella"],
  "movement": {
    "type": "entering",
    "destination": "entrance_hall"
  }
}
```

Or without movement:
```json
{
  "npcs_in_room": ["carla"],
  "movement": {
    "type": null
  }
}
```

## NPC Events (Computed Client-Side)

NPC enter/leave events are **not** returned by the LLM. Instead, they are computed by the engine by comparing the previous quantifier result with the current one:

1. LLM returns `npcs_in_room` (list of NPC IDs present)
2. Engine compares `previous_npcs` vs `current_npcs`
3. `Entered` → NPC in current but not in previous
4. `Left` → NPC in previous but not in current

This delta-based approach avoids requiring the LLM to reason about transitions, making it more reliable than asking for explicit enter/leave events.

## Retry Behavior

If the quantifier returns a **Low confidence** result (e.g., unparseable response), the engine automatically retries the LLM call once. This gives the model a second chance to produce valid JSON. Medium and High confidence results are accepted on the first attempt.

## Customization

The quantifier prompt can be customized at runtime via the **Prompt Presets** tab in the dashboard:

1. Navigate to the **Prompt Presets** tab
2. Create a new Quantifier Prompt preset (or edit an existing non-default one)
3. Click **Set Active** to apply it

The active preset is stored in `AppSettings.active_quantifier_prompt_preset_id`. Its text is assembled via `assemble_prompt_text()` (no global rules or response length for quantifier) and cached in `AppSettings.active_quantifier_prompt`. When an active preset is set, the assembled XML is injected into `QuantifierPromptContext.quantifier_prompt_override` and used by `QuantifierPromptBuilder::build_system_prompt()`. The builder then appends `<available_npc_ids>` and `<available_rooms>` after the assembled preset.

Default presets (shipped as `data/prompt_presets/quantifier/default.json`) are protected and cannot be edited or deleted. To modify a default, create a copy and activate it.

## Code references

- System prompt default: `data/prompt_presets/quantifier/default.json`
- System prompt builder: `src/application/agents/quantifier/prompt.rs:build_system_prompt()` (returns `quantifier_prompt_override` when set)
- User prompt: `src/application/agents/quantifier/prompt.rs:build_user_prompt()`
- Response parsing: `src/application/agents/quantifier/parser.rs` (see `parse_quantifier_response` functions)
- Prompt preset storage: `src/adapters/driven/storage/prompt_preset_storage.rs`
- Dashboard UI: `src/adapters/driving/http/prompt_presets_fragment/`
