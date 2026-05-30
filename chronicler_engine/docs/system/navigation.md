# Specification: Semantic Navigation

> **Related Decisions**: [ADR-006](../adr/adr-006-quantifier-systems.md)

## Objective
The engine's movement system uses quantifier-driven detection. Player types natural language ("I walk through the front gate") and the LLM quantifier detects movement intent from the *narrative outcome*, then the engine resolves the destination room.

## Current Approach: Quantifier-Driven Movement

1. **All Input is FreeAction**: The parser passes all user input to the LLM as FreeAction.
2. **Narrator Response**: The LLM generates a narrative response describing what happens.
3. **Quantifier Detection**: The quantifier analyzes the *narrative outcome* (not just player intent) to determine if the player actually moved. If the narration says the player was blocked or prevented from moving, no movement is detected.
4. **Retry on Low Confidence**: If the quantifier produces an unparseable or uncertain response, it retries once before falling back to static NPCs.
5. **Room Resolution**: The quantifier extracts a destination room name/ID. The engine does a direct lookup in the map. If not found, it creates a dynamic pseudo-room.

### Example Flows
- "I walk through the front gate" → narrator confirms movement → quantifier detects "entering" + "front_gates" → engine resolves to `front_gates` room
- "I head to the kitchen" → narrator confirms movement → quantifier detects "entering" + "kitchen" → engine resolves to `kitchen` room
- "I try to enter but Carla blocks me" → narrator describes blocking → quantifier detects **no movement**

## Resolution Algorithm
1. Quantifier extracts movement from the *narrative outcome* in `<LatestNarration>`
2. If the narration describes the player being blocked, stopped, or prevented from moving → **no movement**
3. System attempts direct room lookup using the extracted destination string as a room ID
4. If no match: creates a dynamic pseudo-room for invalid destinations

### Auto-Trigger Phase

After `attempt_semantic_walk` succeeds, the engine evaluates NPC triggers for the destination room.

**Trigger evaluation:**
- `evaluate_triggers(state)` checks each NPC against `state.npc_encounter_log` (reads `state.movement.current_room_id` internally)
- Matching triggers fire a continuation narration via a second LLM call
- Trigger narrations do NOT cause further movement — the quantifier is skipped for them
- This prevents infinite trigger chains (e.g., trigger causes movement → new trigger fires → ...)

**Quantifier runs twice when triggers fire:**
- First run: after main narration (detects player movement + NPC enter/leave)
- Second run: after trigger continuation (detects NPCs introduced by event text)
- Movement is NOT re-detected for trigger narrations
## Dynamic Room Creation
When `attempt_semantic_walk` fails to find a destination room in the static map, the engine creates a **dynamic (pseudo) room**:
1. `create_dynamic_room(name, description)` generates a placeholder room with a timestamp-based ID
2. The room is stored in `state.movement.dynamic_rooms` for the session
3. `state.movement.current_room_id` is updated to the new room's ID
4. The player can proceed even to invalid destinations
Dynamic rooms are intentionally sparse (no exits, no items) — they serve as fallback containers for player exploration that doesn't map to the static world.
