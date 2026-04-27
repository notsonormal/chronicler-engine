# Plan: NPC Event Layer for Quantifier

## Problem Statement

The quantifier currently tracks **player movement** (entering/leaving rooms), not NPC movement. The TODO item states:

> "It's tracking when the PLAYER is moving in and out of a room. It needs to track when the NPC is entering or leaving. Or has left I suppose."

We need to track when NPCs enter or leave the player's area to:
1. Better manage `times_met` semantics (currently increments on any room entry with NPCs)
2. Support "NPC was here before, now they're gone → they left" inference
3. Enable future features like NPC schedules and multi-room stories

## Proposed Solution: NPC Event Layer

Add a **delta-based event layer** to the quantifier result that explicitly captures which NPCs entered and which left between quantifier calls.

### Changes

#### 1. Add `NpcEvent` types (`src/narrative/quantifier.rs`)

```rust
/// NPC movement event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcEventType {
    Entered,
    Left,
}

/// A single NPC movement event
#[derive(Debug, Clone)]
pub struct NpcEvent {
    pub npc_id: String,
    pub event_type: NpcEventType,
}

/// Events derived from comparing previous vs current NPC presence
#[derive(Debug, Clone, Default)]
pub struct NpcEventList {
    pub events: Vec<NpcEvent>,
    pub confidence: QuantifierConfidence,
}
```

#### 2. Extend `QuantifierResult` with events

```rust
pub struct QuantifierResult {
    pub npcs: QuantifierParseResult,
    pub movement: MovementParseResult,
    pub npc_events: NpcEventList,  // NEW
}
```

#### 3. Add event detection function

In `fragments.rs`, after the second quantifier runs, compute the delta between `previous_room_npcs` and the new `npc_ids`:
- NPCs in new but not in previous → `Entered`
- NPCs in previous but not in new → `Left`

#### 4. Use events to drive `currently_meeting`

- `Entered` → set `currently_meeting = true`
- `Left` → set `currently_meeting = false`

#### 5. Refine `times_met` logic

Only increment `times_met` when an NPC **enters** (not just when they're in the area). This means:
- First meeting (Entered from outside) → increment
- NPC rejoins after leaving → increment (new encounter)
- NPC simply present in area across multiple turns → no increment

## Files to Change

1. **`src/narrative/quantifier.rs`** - Add `NpcEvent`, `NpcEventType`, `NpcEventList`, extend `QuantifierResult`, add event computation
2. **`src/server/fragments.rs`** - Compute NPC events from quantifier delta, use events to update `character_state`
3. **`docs/architecture/system.md`** - Document the NPC event layer
4. **`docs/system/narration_engine.md`** - Document the event-driven NPC tracking

## Implementation Notes

- `Entered` event when NPC transitions from NOT in area → in area
- `Left` event when NPC transitions from in area → NOT in area
- When `Entered` fires, set `currently_meeting = true`
- When `Left` fires, set `currently_meeting = false` AND record that NPC has left (for future "where did they go?" features)
- `times_met` increments ONLY on `Entered` (first encounter or rejoined after leaving)