# System: Auto-Trigger & Reactive Encounters

> **Related Decisions**: [ADR-006](../adr/adr-006-quantifier-systems.md)

The Auto-Trigger system allows the game world to react dynamically to the player's presence based on NPC-specific conditions.

## Overview
When a player enters a room or performs an action, the engine evaluates a set of rules (Triggers) associated with NPCs. If conditions are met, a reactive event (Action) is fired.

## Core Flow

### 1. Player Action
Player performs an action (movement, dialogue, etc.)

### 2. Generate Main Narration
LLM generates the main narrative response

### 4. Second Quantifier (Post-Narration)
The quantifier analyzes the **generated narration** to:
- Detect NPCs that appeared in the narration
- This ensures dynamic NPC appearances (like Gabriella emerging from shadows) are detected

### 5. Times Met Update
For each NPC detected in the narration:
- If `currently_meeting` is false: increment `times_met` and set `currently_meeting = true`
- This tracks encounter cycles: entering → exiting → re-entering

### 6. Trigger Evaluation
`crate::engine::trigger_eval::evaluate_triggers` scans **ALL NPCs** in `state.npcs`, but filters by room:
- Triggers with `room_id: null` (or missing) are **global** — they fire anywhere
- Triggers with `room_id: "some_room_id"` only fire when the player is in that room

This ensures NPC introduction triggers (like Gabriella in the Entrance Hall) don't fire in the wrong location.

### 7. Condition Check
Each trigger is checked against the current `NpcEncounterLog`:
- `TimesMet Eq 0`: Fires on first encounter (times_met is 0 when evaluation happens)
- `TimesMet Gte 1`: Fires on subsequent encounters

### 8. Execution
- If repeatable: Trigger fires and can fire again
- If non-repeatable: Trigger is marked as "fired" and won't re-fire

### 9. Narration
Trigger actions now use the unified 7-layer prompt with continuation context in the user message.

### 10. Inline Event Header
When a trigger fires, the engine stores the event name in `NarrativeState.pending_event`. The next `add_log` call (which adds the trigger continuation narration) absorbs this pending metadata into `LogEntry.event_header`. The frontend renders the event header inside the same div as the continuation narration. There is no standalone `LogType::Event` entry.

## Timing: Evaluate BEFORE Increment

A critical implementation detail: triggers are evaluated BEFORE `times_met` is incremented.

**Why:** If we increment first, `TimesMet Eq 0` would immediately become false, and triggers would never fire.

**The flow:**
1. Second quantifier detects NPCs in narration
2. Evaluate triggers (at this point, times_met still = 0, so TimesMet Eq 0 is TRUE)
3. Trigger fires
4. Increment times_met (now becomes 1)

If step 4 happens before step 2, the trigger would see times_met = 1 and TimesMet Eq 0 would be FALSE.

## Times Met Semantics

The `times_met` counter tracks **unique encounter events** with an NPC. It increments when the quantifier detects an NPC in the room/narration for the first time in that session.

| Scenario | Times Met Increments? |
| :--- | :--- |
| Player enters room with NPC already there | Yes - quantifier detects NPC |
| NPC follows player to new room | Yes - quantifier detects NPC in new room |
| NPC appears in narration while player is in room | Yes - quantifier detects NPC in narration |
| Player stays in room with same NPC | No - already currently_meeting |
| Player returns to room with same NPC | Yes - re-entry after leaving |

The key variable is `currently_meeting`:
- Set to `true` when quantifier first detects NPC in the current room session
- Set to `false` when player enters a new room (different from last room)
- `times_met` only increments when `currently_meeting` was `false`

## Trigger Conditions
| Condition | Description |
| :--- | :--- |
| `TimesMet` | Evaluates the `times_met` counter using `Eq`, `Lt`, or `Gte`. |
| `HasItem` | (Planned) Checks player inventory for a specific item ID. |

## Trigger Room Scoping

By default, triggers are **global** — they fire regardless of where the player is. To restrict a trigger to a specific room, add `room_id`:

```json
{
  "condition": {"TimesMet": ["Eq", 0]},
  "action": {"name": "Gabriella Introduction", "narration_prompt": "Gabriella emerges from the shadows..."},
  "repeat": false,
  "room_id": "entrance_hall"
}
```

This trigger only fires when `state.movement.current_room_id == "entrance_hall"`.

## Trigger Actions
| Action | Description |
| :--- | :--- |
| `Narrate` | Appends a custom LLM prompt to the arrival/action narration. |

### Action Fields
```json
{
    "name": "Event Name",
    "narration_prompt": "LLM prompt text"
}
```

- `name` (required): Display name for the event. When the trigger fires, an event header with this name appears in the story log before the narration.
- `narration_prompt` (required): The prompt sent to the LLM to generate continuation narration.

## Character State Tracking
The `NpcEncounterLog` struct tracks encounter state:

```rust
pub struct NpcEncounterState {
    pub times_met: u32,           // Number of completed encounter cycles
    pub trigger_fired: HashMap<usize, bool>,  // Non-repeatable triggers fired
    pub currently_meeting: bool,  // Player is currently in the same room
}
```

- `times_met`: Incremented when player enters room with NPC (after leaving)
- `currently_meeting`: Set to true on room entry, false on room exit
- `trigger_fired`: Indices of non-repeatable triggers that have fired

## Event Headers

When a trigger fires, if its action has a `name`, the engine inserts an **event header** entry into the story log before the LLM-generated narration. Event headers are visually distinct from location headers (room names) and use a blue/cyan color (`#38bdf8`).

Example story log output:
```
─── Entrance Hall ─── 10:42
You step into the grand hall.

─── Gabriella Introduction ─── 10:43
Gabriella emerges from the shadows...
```

Event headers:
- Have no edit or retry buttons
- Are rendered with `.event-header` and `.event-timestamp` CSS classes

## Key Design Decisions

1. **Room-Aware Evaluation**: Triggers are checked for ALL loaded NPCs, but filtered by `room_id`. Global triggers (no `room_id`) fire anywhere. Room-scoped triggers only fire in their designated room. This prevents introduction triggers from firing in the wrong location while still supporting dynamic appearances.

2. **Quantifier Runs After Narration and After Triggers**: The quantifier runs once after main narration generation (detects NPCs and movement), and again after trigger continuation if a trigger fired (detects NPCs introduced by event text).

3. **Times Met vs Trigger Fire**: `times_met` is incremented based on movement/room entry, NOT when triggers fire. This prevents the bug where trigger fires would increment the counter for the next evaluation.

4. **Named Triggers**: Every trigger action requires a `name`. This name is used for the event header and helps players recognize important story moments at a glance.

---

## Mutation Order Invariant

The action pipeline and `execute_freeaction_impl` in `src/engine/action_processing.rs` mutate state in a strict, load-bearing order. Steps 4b and 4c happen in the application pipeline (`ActionPipeline`), not inside the engine function:

| Step | Operation | Why it must come here |
| :--- | :--- | :--- |
| 1 | `handle_movement()` — may update `movement.current_room_id` | Room must be current before NPCs are resolved |
| 2 | Resolve current NPCs from quantifier result | Uses updated `movement.current_room_id` from step 1 |
| 3 | `state.add_log(narration_text)` | Narration must be in history before triggers read it |
| 4a | `evaluate_triggers()` + build prompt | Reads `state.narrative.history()` (step 3) to build the trigger continuation prompt |
| 4b | Trigger LLM call | Runs in `ActionPipeline::phase_trigger_continuation`, outside the state lock |
| 4c | `commit_trigger_narration()` | Runs in `ActionPipeline`, re-acquires lock to add trigger logs and mark trigger fired |
| 5 | `apply_npc_events()` — mutates `npc_encounter_log` | `times_met` increments AFTER trigger evaluation (see Timing section above) |

**What breaks if you change the order:**

- Swapping steps 3 and 4a: triggers generate continuation without seeing the current narration as context — the LLM has no story thread to continue from.
- Swapping steps 4a and 5: `times_met` increments before trigger evaluation — `TimesMet Eq 0` would never fire on first encounter.
- Moving step 1 after step 3: the narration gets logged against the old room, then the room changes — state is inconsistent.
- Moving step 4b inside the lock: frontend cannot poll the main narration until the trigger LLM completes — both texts appear simultaneously.

This invariant is enforced by code structure, not by runtime checks. If you refactor `execute_freeaction_impl`, preserve this order explicitly.
