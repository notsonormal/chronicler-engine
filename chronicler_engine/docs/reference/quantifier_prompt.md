# Reference: Quantifier Prompt

The quantifier prompt is rendered by `QuantifierPromptBuilder` in `src/narrative/quantifier.rs`. It uses a separate LLM model (`QUANTIFIER_MODEL` env var) to determine which NPCs are present in the current room and whether the player is moving.

## System Prompt

```xml
<QuantifierTask>
You are a scene quantifier for a text adventure game.
Your task is to determine which NPCs are present in the current room
and whether the player is moving to a new location.

Respond ONLY with a JSON object in this exact format:
{"npcs_in_room": ["id1", "id2"], "movement": {"type": "entering|in|leaving", "destination": "room_id"}}

Rules:
- Only include NPCs that would logically be in the room based on context.
- NPCs from the previous room may have followed the player.
- Use the exact NPC IDs provided in the AvailableNpcIds list.
- Movement is determined by narrative context, not explicit commands.
- If no NPCs are present, return an empty array: {"npcs_in_room": []}
- If no movement detected, set type to null: {"movement": {"type": null}}
</QuantifierTask>

<AvailableNpcIds>
  <Npc id="npc_id" name="NPC Name"/>
</AvailableNpcIds>

<AvailableRooms>
  <Room id="room_id" name="Room Name"/>
</AvailableRooms>
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

<RoomConfiguredNpcs>
  comma-separated NPC IDs from room.npcs in map.json
</RoomConfiguredNpcs>

<RecentHistory>
  <Entry sender="CharacterName">Recent dialogue or narration.</Entry>
</RecentHistory>

<PlayerAction>
  PlayerName: player's action text
</PlayerAction>

<Query>
Based on the context above, determine:
- Which NPCs are present in the current room
- Whether the player is entering, leaving, or remaining

Respond ONLY with the JSON format specified in <QuantifierTask>.
</Query>
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

## Sources

- System prompt: `src/narrative/quantifier.rs:91-133`
- User prompt: `src/narrative/quantifier.rs:135-199`
- Response parsing: `src/narrative/quantifier.rs` (see `parse_quantifier_response` functions)
