# Plan: Trigger Identity: Index → UUID

## Problem

Triggers are identified by array index (`usize`) in `NpcEncounterState.trigger_fired`. If an NPC's trigger array is reordered or has triggers inserted mid-game, stale indices corrupt the HashMap.

**Current:**
```rust
pub struct NpcEncounterState {
    pub trigger_fired: HashMap<usize, bool>,  // INDEX, not stable
}
```

## Goal

Assign a stable UUID to each `Trigger` at parse time and use UUID as the key in `NpcEncounterState.trigger_fired`.

---

## Task Breakdown

| # | Task | Size | Files |
|---|------|------|-------|
| 1 | Update `Trigger` struct and `NpcEncounterState` | XS | `src/model/trigger.rs` |
| 2 | Update `StoredTriggerContext` | XS | `src/model/state.rs` |
| 3 | Update `TriggerMatch` | XS | `src/engine/action_processing.rs` |
| 4 | Update `trigger_eval` functions | S | `src/engine/trigger_eval.rs` |
| 5 | Update `pipeline.rs` | XS | `src/application/action_pipeline/pipeline.rs` |
| 6 | Add UUID assignment at parse time | S | `src/bootstrap/load.rs` |
| 7 | Update JSON schema | XS | `data/schemas/character.schema.json` |
| 8 | Fix inline Trigger tests | S | `tests/*` (4 files) |

---

## Changes

### 1. `src/model/trigger.rs`
```rust
pub struct Trigger {
    pub id: Uuid,  // NEW: stable identity
    pub requirement: TriggerRequirement,
    pub narration: TriggerNarration,
    pub repeat: bool,
    pub room_id: Option<String>,
}

pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<Uuid, bool>,  // usize → Uuid
    pub currently_meeting: bool,
}
```

### 2. `src/model/state.rs`
```rust
pub struct StoredTriggerContext {
    pub npc_id: String,
    pub trigger_id: Uuid,  // trigger_idx → trigger_id
    // ... rest unchanged
}
```

### 3. `src/engine/action_processing.rs`
```rust
pub struct TriggerMatch {
    pub npc_id: String,
    pub trigger_id: Uuid,  // usize → Uuid
    // ... rest unchanged
}
```

### 4. `src/engine/trigger_eval.rs`
- `evaluate_triggers`: Return `Vec<(NpcCard, Trigger, Uuid)>` instead of `usize`
- Use `trigger.id` instead of enumerate index
- `mark_trigger_fired(trigger_id: Uuid)`
- `is_trigger_fired(trigger_id: Uuid)`

### 5. `src/application/action_pipeline/pipeline.rs`
- Line ~641: `trigger_id: trigger_match.trigger_id`

### 6. `src/bootstrap/load.rs`
```rust
fn assign_trigger_ids(npc: &mut NpcCard) {
    for trigger in &mut npc.triggers {
        if trigger.id == Uuid::nil() {
            trigger.id = Uuid::new_v4();
        }
    }
}
```

### 7. `data/schemas/character.schema.json`
```json
"id": { "type": "string", "format": "uuid" }
```

### 8. Test files (4)
- Add `id: Uuid::nil()` to all inline `Trigger { ... }` constructions

---

## Backward Compatibility

1. **Trigger.id**: `#[serde(default)]` with `Uuid::nil()` default — old JSON without `id` works
2. **NpcEncounterState.trigger_fired**: Breaking change — old saves with `usize` keys will fail to deserialize

---

## Verification

1. `cargo build` — no compilation errors
2. `cargo test` — all tests pass
3. Manual: Load a character card with triggers, verify UUIDs assigned
