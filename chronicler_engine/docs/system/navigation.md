# Specification: Semantic Navigation

> **Related Decisions**: [ADR-006](../adr/adr-006-quantifier-systems.md)

## Objective
The engine's movement system uses quantifier-driven detection. Player types natural language ("I walk through the front gate") and the LLM quantifier detects movement intent, then validates against semantic exits in map.json.

## Current Approach: Quantifier-Driven Movement

1. **All Input is FreeAction**: The parser passes all user input to the LLM as FreeAction.
2. **Narrator Response**: The LLM generates a narrative response describing what happens.
3. **Quantifier Detection**: The quantifier analyzes the *narrative outcome* (not just player intent) to determine if the player actually moved. If the narration says the player was blocked or prevented from moving, no movement is detected.
4. **Retry on Low Confidence**: If the quantifier produces an unparseable or uncertain response, it retries once before falling back to static NPCs.
5. **Semantic Exits**: Rooms define semantic exits with triggers and keywords for natural language matching.

### Example Flows
- "I walk through the front gate" → narrator confirms movement → quantifier detects "entering" + "front gate" trigger
- "I head to the kitchen" → narrator confirms movement → quantifier detects "entering" + "kitchen" trigger (via keywords)
- "I try to enter but Carla blocks me" → narrator describes blocking → quantifier detects **no movement**

## Semantic Exit Format
Rooms in map.json define semantic exits:
```json
{
  "semantic_exits": [
    {
      "trigger": "front gate",
      "destination": "entrance_hall",
      "keywords": ["enter", "go through", "pass through"]
    }
  ]
}
```

## Resolution Algorithm
1. Quantifier extracts movement from the *narrative outcome* in `<LatestNarration>`
2. If the narration describes the player being blocked, stopped, or prevented from moving → **no movement**
3. System matches trigger text against current room's semantic exits
4. Keywords enable flexible matching ("go through the front gate" matches "go through")
5. If no match: creates dynamic pseudo-room for invalid destinations

### Auto-Trigger Phase

After `attempt_semantic_walk` succeeds, the engine evaluates NPC triggers for the destination room.

**Trigger evaluation:**
- `evaluate_triggers(state, room_id)` checks each NPC in the room against `state.npc_encounter_log`
- Matching triggers fire a continuation narration via a second LLM call
- Trigger narrations do NOT cause further movement — the quantifier is skipped for them
- This prevents infinite trigger chains (e.g., trigger causes movement → new trigger fires → ...)

**Quantifier skip for triggers:**
- Movement is NOT re-detected for trigger narrations
- The quantifier only runs once per player action (after the initial arrival narration)
- This ensures trigger responses don't cascade into additional movement
