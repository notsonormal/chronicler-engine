# Phase 5: Module Mental Models

**Date:** 2026-05-30  
**Scope:** For each major module, document what an AI needs to know but isn't obvious from reading code alone  
**Method:** Subagent-based full-file analysis + invariant extraction

---

## Executive Summary

Created mental model sheets for 4 critical modules. These documents capture:

1. **What problem the module solves**
2. **Load-bearing invariants** (must preserve or system breaks)
3. **Side effects** (observable changes beyond return values)
4. **Misleading names** (names that don't match actual meaning)
5. **Load-bearing vs incidental code** (what can be changed safely)
6. **AI modification checklist** (what to verify before changing)

**Critical finding:** The `execute_freeaction_impl` mutation order is the most load-bearing invariant in the entire codebase. Breaking it silently breaks the trigger system.

---

## 1. Module: `src/engine/action_processing.rs`

### What Problem Does This Solve?

Processes a free-action turn end-to-end: receives quantifier result + narration text, applies movement, logs narration, evaluates triggers, updates NPC encounter state, returns next state + trigger match.

**The bridge:** Between the quantifier (player intent parser) and the game state.

### Load-Bearing Invariants

| Invariant | Why | Violation Impact |
|-----------|-----|------------------|
| **Mutation order: handle_movement → add_log → evaluate_triggers → apply_npc_events** | Triggers must see narration in history; times_met must be 0 when triggers fire | Triggers never fire (`TimesMet Eq 0` broken) |
| **Narration logged BEFORE trigger evaluation** | Trigger continuation prompts read history for context | Triggers have no story thread to continue |
| **times_met incremented AFTER trigger evaluation** | `TimesMet Eq 0` condition checks pre-increment value | Triggers miss first encounter |
| **`set_currently_meeting(true)` on room entry** | `apply_npc_events` only increments on Entered when `currently_meeting` was false | times_met doesn't track first encounter |
| **Dynamic room created on semantic walk failure** | Invalid destinations become pseudo-rooms | Unhandled errors, no fallback |

### Side Effects

| Side Effect | Location | Impact |
|-------------|----------|--------|
| `assert_state_consistency()` called after every mutation | Lines 78, 97, 123, 140, 173 | Runtime panic if state inconsistent |
| `set_currently_meeting` called in `handle_movement` for NPCs in destination | Line 70 | Affects encounter tracking before main quantifier events |
| `create_dynamic_room` inserts into `state.movement.dynamic_rooms` | Line 64 | Persistent dynamic room until game reset |
| `state.narrative.pending_location` set in `handle_movement` | Line 75 | Used for UI location headers |

### Misleading Names

| Name | Actual Meaning | Risk |
|------|----------------|------|
| `TurnResult.trigger_match` | Only populated if trigger fires, null otherwise | Low — name is accurate |
| `FreeActionContext.narration_text` | The LLM-generated narration, not player input | Low — context clarifies |
| `apply_npc_events` | Computes NpcEvent list from diff, THEN applies | Medium — function name doesn't mention compute |

### Load-Bearing vs Incidental

**Load-bearing (do not change without understanding invariants):**
- The entire `execute_freeaction_impl` function body (lines 128-180)
- The mutation order in that function
- `apply_npc_events` logic
- `commit_trigger_narration` logic

**Incidental (can change without breaking invariants):**
- Adding additional fields to `FreeActionContext` (as long as consumed correctly)
- Adding more diagnostic logging
- Renaming internal helper variables

### AI Modification Checklist

Before modifying `execute_freeaction_impl`:
- [ ] Read `docs/system/triggers.md` section "Mutation Order Invariant"
- [ ] Verify mutation order is preserved: handle_movement → add_log → evaluate_triggers → apply_npc_events
- [ ] If adding a new step, place it in the correct order
- [ ] If changing NPC processing, verify times_met is still incremented AFTER trigger eval
- [ ] Run `cargo nextest run logic_tests` to verify trigger behavior
- [ ] Run `cargo nextest run flow_mock` to verify end-to-end flow

---

## 2. Module: `src/application/action_pipeline/pipeline.rs`

### What Problem Does This Solve?

Orchestrates the full player-input-to-world-state cycle across 6 phases:
1. Snapshot (save state before LLM call)
2. Narrate (LLM generates main prose)
3. Post-generation quantifier (detects NPCs + movement)
4. Engine commit (applies state to game state)
5. Optional trigger continuation (second LLM call if trigger fires)
6. Post-trigger reconcile (second quantifier run if trigger fired)
7. Finalize (save result)

**The coordinator:** Wires together LLM backends, quantifier agents, and engine logic.

### Load-Bearing Invariants

| Invariant | Why | Violation Impact |
|-----------|-----|------------------|
| **Input buffer status/phase tracks progress** | UI polls status endpoint; wrong state = broken UI | UI shows wrong phase |
| **Snapshots before every LLM call** | Crash recovery; retry reloads from snapshot | Data loss on crash |
| **Cancel checks after every LLM call** | Graceful abort on stale requests | Wasted LLM calls |
| **Phase=Quantifying convention in reconcile** | Second quantifier sets phase to Quantifying | UI shows wrong phase |

### Hidden Invariants (Duplicated Logic)

| Duplicated Pattern | Risk | Location |
|-------------------|------|----------|
| `save_message_and_snapshot` calls | Both `run_from_input` and `run_trigger_continuation` must save | Lines 155, 203, 294, 350, 420 |
| Error handling (map_llm_error) | Both entry points use same error mapping | Lines 166, 209, 298, 352 |
| Cancel check placement | Must be after LLM call, before continue | Lines 172, 215, 304, 356 |

### Side Effects

| Side Effect | Location | Impact |
|-------------|----------|--------|
| `save_early_error` reloads state from storage | Not from parameter | If storage has newer state, overwrites |
| `save_message_and_snapshot` is load-bearing | Crash recovery depends on it | Removing breaks recovery |
| `GenerationGuard` RAII cleanup | Sets status to Idle on drop | Must not panic in drop |

### Duplicated Logic Analysis

`run_from_input` and `run_trigger_continuation` share most logic. Key differences:

| Phase | run_from_input | run_trigger_continuation |
|-------|---------------|--------------------------|
| Quantifier | Full run (NPCs + movement) | Full run (NPCs + movement) |
| Engine | execute_freeaction_impl | execute_freeaction_impl |
| Trigger | Optional (if trigger_match) | Required (uses stored context) |
| Reconcile | Optional (if trigger fired) | Required |

**Risk:** If one is updated but not the other, invariants diverge.

### AI Modification Checklist

Before modifying `pipeline.rs`:
- [ ] Read both `run_from_input` and `run_trigger_continuation`
- [ ] Verify any change is applied to both functions (or intentional divergence)
- [ ] Check cancel check placement (must be after LLM call)
- [ ] Verify snapshot saves happen before every LLM call
- [ ] Run `cargo nextest run flow_mock` to verify end-to-end flow
- [ ] Verify phase values match `GenerationPhase` enum

---

## 3. Module: `src/narrative/agents/quantifier/`

### What Problem Does This Solve?

Analyzes LLM-generated narration to detect:
1. **NPCs present** in the scene (from narration text)
2. **Movement intent** (player destination from narration outcome)

**Dual role:** Acts as both an `Agent` trait implementation AND a parser of LLM output.

### Module Structure

```
quantifier/
├── mod.rs        ─── Re-exports all types
├── agent.rs      ─── Agent trait implementation (QuantifierAgent)
├── core.rs       ─── LLM call orchestration (quantify_room_with_llm_call)
├── parser.rs     ─── LLM response parsing (JSON + text fallback)
├── prompt.rs     ─── Prompt construction (QuantifierPromptBuilder)
└── types.rs      ─── Prompt context types + re-exports
```

### Load-Bearing Invariants

| Invariant | Why | Violation Impact |
|-----------|-----|------------------|
| **Retry logic: 2 attempts before fallback** | LLM parsing can fail; second try often succeeds | Unnecessary fallbacks |
| **JSON-first parsing, text fallback** | Model may not return valid JSON | Parser errors |
| **Movement detection skipped in second run** | Trigger continuations shouldn't re-detect movement | Broken navigation |
| **Static fallback when no LLM** | Tests use MockBackend without LLM calls | Tests break |
| **Fallback NPCs from room config** | When quantifier fails, use static NPCs from map | Scene not empty |

### Dual Role: Agent vs Parser

**Agent role (`agent.rs`):**
- `QuantifierAgent` implements `Agent` trait
- `execute()` calls `determine_npcs_in_room()`
- Integrates with `AgentRegistry` for post-generation agents

**Parser role (`parser.rs`):**
- `parse_quantifier_response_with_movement()` parses LLM output
- `extract_npcs()` handles NPC extraction
- Two parsing strategies: JSON (primary) + text (fallback)

**Confusion point:** The module is called "quantifier" but it's actually two distinct concerns:
1. The agent that runs the quantifier logic
2. The parser that interprets quantifier results

### NPC Detection Strategy

```rust
// From core.rs:quantify_room_with_llm_call
let known_ids: Vec<String> = context.all_known_npcs.iter().map(|n| npc.id.clone()).collect();

// Strategy: Send all known NPCs, LLM picks which are present
// Result: List of present NPCs, not list of all NPCs
```

**Important:** The quantifier receives ALL known NPCs, returns only those present. Unknown NPCs cannot be detected.

### Side Effects

| Side Effect | Location | Impact |
|-------------|----------|--------|
| Retry on parsing failure | core.rs:41-96 | Extra LLM calls on failure |
| Logging on parse failure | core.rs:83 | Debug info in logs |
| Fallback to static NPCs | core.rs:101-112 | Scene populated from map config |

### AI Modification Checklist

Before modifying the quantifier module:
- [ ] Understand dual role (agent + parser)
- [ ] Verify retry logic (2 attempts)
- [ ] Check second run skips movement detection
- [ ] Test with MockBackend (no LLM needed)
- [ ] Verify fallback behavior when LLM fails
- [ ] Run `cargo nextest run quantifier` tests

---

## 4. Module: `src/model/state.rs` (GameState + Sub-states)

### What Problem Does This Solve?

Holds the complete game state:
- World (lore, rules)
- Map (rooms, regions)
- Player (identity, inventory)
- NPCs (definitions, encounter state)
- Movement (current room, dynamic rooms)
- Narrative (history, generation status)
- Scene (current NPCs, confidence)
- Triggers (stored context for retry)

### State Hierarchy

```
GameState (aggregate root)
├── world: WorldCard ─────────── lore, rules, starting scenario
├── map: MapDef ──────────────── rooms, regions, directions
├── player: PlayerCard ──────── identity, inventory
├── npcs: HashMap<String, NpcCard> ─── NPC definitions
├── movement: MovementState ─── current_room_id, dynamic_rooms
├── narrative: NarrativeState ── history, generation status, input buffer
├── scene: SceneState ────────── npcs_in_area, confidence
├── npc_encounter_log: NpcEncounterLog ─── per-NPC encounter tracking
└── builder: GameStateBuilder ── fluent construction
```

### Sub-state Responsibilities

| Sub-state | Responsibility | Lives In |
|-----------|---------------|----------|
| `MovementState` | Player location, dynamic rooms | `state.rs` |
| `NarrativeState` | Message history, generation status, input buffer, last trigger | `state.rs` |
| `SceneState` | Current NPCs in area, quantifier confidence | `state.rs` |
| `NpcEncounterLog` | Per-NPC: times_met, trigger_fired, currently_meeting | `trigger.rs` |
| `StoredTriggerContext` | Last trigger + LLM params for retry | `state.rs` |

### Load-Bearing Invariants

| Invariant | Why | Violation Impact |
|-----------|-----|------------------|
| **`npc_encounter_log` keyed by NPC ID, not NPC index** | Consistent lookup across turns | Triggers break |
| **`current_room_id` must match a room in map** | Navigation relies on this | Broken navigation |
| **Messages have snapshot_id** | Retry loads state from snapshot | Retry breaks |
| **`active_swipe_index` within bounds** | UI relies on this for navigation | UI crash |
| **`generation.status` and `generation.phase` stay consistent** | UI polls these | Broken UI polling |

### NpcEncounterState Naming Issue

**Location:** `src/model/trigger.rs`

```rust
pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<usize, bool>,
    pub currently_meeting: bool,
}
```

**Problem:** Name implies "a single character's state" but it's actually "per-NPC encounter tracking".

**Impact:** Every developer must mentally correct "NpcEncounterState" → "per-NPC encounter tracker".

**Recommendation:** Rename to `NpcEncounterState` is already clear. The issue is the module (`trigger.rs`) where it lives. Consider moving to `state.rs`.

### GameStateBuilder Pattern

```rust
let state = GameStateBuilder::new(world, map, player, npcs)
    .with_initial_room(starting_room_id)
    .build();

// New fields get Default::default() automatically
```

**Invariant:** Builder must handle missing fields gracefully. New fields added to `GameState` should not break existing code.

### Side Effects

| Side Effect | Location | Impact |
|-------------|----------|--------|
| `add_log` increments `next_log_id` | `state.rs` | Log IDs monotonically increase |
| `add_log` sets `pending_location` | `state.rs:77` | Used for UI location headers |
| `npc_encounter_log` updated by `apply_npc_events` | `action_processing.rs:83` | Affects trigger conditions |

### AI Modification Checklist

Before modifying GameState:
- [ ] Understand sub-state responsibilities
- [ ] Verify any new field has Default impl
- [ ] Update GameStateBuilder if adding required fields
- [ ] Check if snapshot needs updating (exclude new field or include?)
- [ ] Run `cargo nextest run logic_tests` for state consistency
- [ ] Verify arch-lint still passes (model must not depend on outer tiers)

---

## 5. Summary: AI Must-Know Cards

### Card 1: `execute_freeaction_impl` (action_processing.rs)

```
┌─────────────────────────────────────────────────────────────┐
│ MODULE: execute_freeaction_impl                              │
│ RISK: CRITICAL                                              │
├─────────────────────────────────────────────────────────────┤
│ WHAT: Turn processing — narrate → trigger → update          │
│                                                             │
│ INVARIANTS (preserve these or system breaks):               │
│  1. handle_movement → update room BEFORE anything           │
│  2. add_log(narration) → BEFORE trigger eval                │
│  3. evaluate_triggers → AFTER log, BEFORE times_met++       │
│  4. apply_npc_events → times_met++ AFTER trigger eval      │
│                                                             │
│ VIOLATION: Triggers never fire (TimesMet Eq 0 broken)       │
│                                                             │
│ READ: docs/system/triggers.md section "Mutation Order"       │
│ TEST: cargo nextest run logic_tests                         │
└─────────────────────────────────────────────────────────────┘
```

### Card 2: `ActionPipeline` (pipeline.rs)

```
┌─────────────────────────────────────────────────────────────┐
│ MODULE: ActionPipeline                                       │
│ RISK: HIGH                                                  │
├─────────────────────────────────────────────────────────────┤
│ WHAT: Orchestrates LLM → quantifier → engine → trigger      │
│                                                             │
│ INVARIANTS:                                                 │
│  1. Input buffer status/phase tracks progress (UI polls)    │
│  2. Snapshot before every LLM call (crash recovery)         │
│  3. Cancel check after every LLM call (graceful abort)      │
│  4. Phase=Quantifying in reconcile (UI convention)          │
│                                                             │
│ DUPLICATED LOGIC: run_from_input and run_trigger_continuation│
│ must be updated together                                    │
│                                                             │
│ VIOLATION: Broken UI polling, lost state on crash           │
│                                                             │
│ TEST: cargo nextest run flow_mock                           │
└─────────────────────────────────────────────────────────────┘
```

### Card 3: Quantifier (narrative/agents/quantifier/)

```
┌─────────────────────────────────────────────────────────────┐
│ MODULE: Quantifier Agent                                     │
│ RISK: MEDIUM                                                │
├─────────────────────────────────────────────────────────────┤
│ WHAT: Detects NPCs + movement from LLM narration            │
│                                                             │
│ DUAL ROLE:                                                  │
│  - Agent: implements Agent trait, runs in post-gen phase    │
│  - Parser: parses LLM JSON/text response                    │
│                                                             │
│ INVARIANTS:                                                 │
│  1. Retry 2x before fallback                                │
│  2. JSON-first parsing, text fallback                       │
│  3. Second run skips movement detection (trigger continuations)│
│  4. Fallback uses room config NPCs when LLM fails           │
│                                                             │
│ VIOLATION: Unnecessary fallbacks, broken navigation          │
│                                                             │
│ TEST: cargo nextest run quantifier tests                     │
└─────────────────────────────────────────────────────────────┘
```

### Card 4: GameState (model/state.rs)

```
┌─────────────────────────────────────────────────────────────┐
│ MODULE: GameState + Sub-states                               │
│ RISK: HIGH                                                  │
├─────────────────────────────────────────────────────────────┤
│ WHAT: Complete game state aggregate                          │
│                                                             │
│ SUB-STATES:                                                 │
│  - MovementState: player location                            │
│  - NarrativeState: history, generation, input               │
│  - SceneState: current NPCs, confidence                     │
│  - NpcEncounterLog: per-NPC encounter tracking               │
│                                                             │
│ KEY INVARIANTS:                                             │
│  1. npc_encounter_log keyed by NPC ID (not index)           │
│  2. current_room_id must match room in map                  │
│  3. Messages have snapshot_id (for retry)                   │
│  4. active_swipe_index within bounds                        │
│                                                             │
│ VIOLATION: Broken triggers, navigation, retry, or UI         │
│                                                             │
│ TEST: cargo nextest run logic_tests                         │
└─────────────────────────────────────────────────────────────┘
```

---

*Phase 5 complete. Proceeding to Phase 6: Test Coverage for Invariants.*