# Phase 4: Documentation-Code Consistency Check

**Date:** 2026-05-30  
**Scope:** Verify key documents match implementation  
**Method:** Subagent-based comparison of docs against code

---

## Executive Summary

Found **16 discrepancies** across 3 document comparisons:
- 3 HIGH severity (incorrect/misleading documentation)
- 8 MEDIUM severity (incomplete documentation)
- 5 LOW severity (minor issues)

**Critical finding:** `docs/system/game_flow.md` describes Phase 4.5 NPC processing order incorrectly — the doc says one thing, the code does another.

---

## 1. game_flow.md vs pipeline.rs

### Discrepancies Found: 6 total

| # | Severity | Location | Documentation Says | Code Does |
|---|----------|----------|-------------------|-----------|
| 1 | HIGH | Phase 4.5 | "Process movement intent" → "If moved: update room state (no additional LLM call)" | `handle_movement` is called BEFORE quantifier, not after |
| 2 | HIGH | Phase 4.5 | Quantifier detects movement and NPC Enter/Leave in same call | Movement detection and NPC detection are separate operations in `QuantifierResult` |
| 3 | MEDIUM | Phase 5 | "evaluate_triggers(state) — first match only (inside lock)" | `evaluate_triggers` is called inside `execute_freeaction_impl`, not in pipeline |
| 4 | MEDIUM | Phase 5 | "Cancellation checkpoint — aborts before second LLM call" | Cancellation is checked in `ActionPipeline`, not in `evaluate_triggers` |
| 5 | MEDIUM | Retry Flow | "Find last Input message, load its snapshot_id" | Retry logic finds last message with `snapshot_id`, not specifically Input |
| 6 | LOW | General | "7-layer prompt" | Actually 8-layer system (0-7 + safety margin "Phi") |

### Detailed Analysis

#### Discrepancy #1: Phase 4.5 NPC Processing Order (HIGH)

**Documented flow (game_flow.md):**
```
Phase 4: Main LLM Narration
    ↓
Phase 4.5: Quantifier & Movement
    1. Post-narration Quantifier analyzes
    2. Process movement intent
    3. If moved: update room state (no additional LLM call — arrival is part of main narration)
    4. Determine NPC Enter/Leave events
```

**Actual flow (action_processing.rs + pipeline.rs):**
```
ActionPipeline::run_from_input
    ↓
Narrator LLM call (main narration)
    ↓
First quantifier run (detects movement + NPCs)
    ↓
movement applied via handle_movement() ← BEFORE trigger eval
    ↓
Trigger evaluation (evaluate_triggers)
    ↓
Trigger continuation LLM call (if trigger fires)
    ↓
Second quantifier run (detects NPCs from trigger text, NOT movement)
```

**Impact:** The doc says "update room state (no additional LLM call)" as part of Phase 4.5, implying movement is processed after the quantifier. The actual code processes movement during `execute_freeaction_impl`, which is called BEFORE the quantifier result is fully consumed.

**Note:** This may be a timing issue in how the doc describes the flow vs. how it actually executes. The doc describes the conceptual flow; the code implements it slightly differently.

#### Discrepancy #2: Quantifier Dual Output (HIGH)

**Documented (game_flow.md Phase 4.5):**
> "Determine NPC Enter/Leave events"

**Actual (QuantifierResult):**
```rust
pub struct QuantifierResult {
    pub npcs: QuantifierParseResult,      // NPC presence
    pub movement: MovementParseResult,    // Movement destination + type
    pub confidence: QuantifierConfidence,
}
```

**NpcEventList** is computed separately via `compute_npc_events()` in `action_processing.rs`, diffing previous vs. current NPCs. The quantifier itself only returns `npcs` + `movement`, not `NpcEvent` list.

**Impact:** The doc implies the quantifier produces NPC events. The quantifier produces NPC IDs + movement; the engine computes events by diffing.

#### Discrepancy #3: evaluate_triggers Location (MEDIUM)

**Documented (game_flow.md Phase 5):**
> "evaluate_triggers(state) — first match only (inside lock)"

**Actual:** `evaluate_triggers` is called inside `execute_freeaction_impl` (engine tier), not in the application pipeline. The lock context is different.

**Impact:** Low — the doc describes intent correctly but the location is imprecise.

---

## 2. system.md vs Actual Module Structure

### Discrepancies Found: 2 total

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | HIGH | Line 204-205 | Documents `server_helpers` module with `create_app_for_testing` functions — **module does not exist** |
| 2 | LOW | Line 203, 205 | `test_app_builder` listed twice |

### Detailed Analysis

#### Discrepancy #1: server_helpers Module Missing (HIGH)

**Documented in `docs/architecture/system.md` (Tier 10):**
> "`server_helpers`: `create_app_for_testing`, `create_app_for_testing_with_settings`"

**Actual:** No `server_helpers.rs` file exists. Search confirmed these functions don't exist anywhere in `src/test_support/`.

**Impact:** An AI reading the architecture doc would try to use `server_helpers` and find it doesn't exist. The actual test helpers are in `test_support/fixtures.rs` and `test_support/context.rs`.

**Fix:** Remove `server_helpers` from the docs, or implement the module if it was intended.

#### Discrepancy #2: Duplicate test_app_builder Entry (LOW)

**Documented in `docs/architecture/system.md` (Tier 10):**
> Line 203: `test_app_builder`: Fluent test app builder API  
> Line 205: `test_app_builder`: (again, under server_helpers description)

**Actual:** `test_app_builder` exists in `src/test_support/test_app_builder.rs`.

**Impact:** Minor — just a duplicate entry. Easy fix.

---

## 3. navigation.md vs logic.rs

### Discrepancies Found: 8 total (all MEDIUM)

| # | Location | Documentation Says | Actual Behavior |
|---|----------|-------------------|----------------|
| 1 | Quantifier runs | "Quantifier runs twice when triggers fire" | Correct — first (movement + NPC) then second (NPC only) |
| 2 | Movement re-detection | "Movement is NOT re-detected for trigger narrations" | Correct — second quantifier skips movement |
| 3 | Trigger continuation | "Trigger narrations do NOT cause further movement" | Correct — no second movement LLM call |
| 4 | Trigger eval scope | "evaluate_triggers checks each NPC against npc_encounter_log" | Partially correct — also checks room_id constraint |
| 5 | Room resolution | "Engine does a direct lookup in the map" | Only partially — fallback to dynamic room creation |
| 6 | "Affects scene" | Doc uses term "affects scene" without defining | No explicit definition of "scene" |
| 7 | Dynamic room creation | Not documented | `create_dynamic_room` in `handle_movement` creates pseudo-rooms |
| 8 | NPC resolution order | Not documented | `handle_movement` → `attempt_semantic_walk` → direct lookup → fallback |

### Detailed Analysis

#### Discrepancy #5: Dynamic Room Fallback Not Documented (MEDIUM)

**Documented (navigation.md):**
> "System attempts direct room lookup using the extracted destination string as a room ID"

**Actual (logic.rs:attempt_semantic_walk + action_processing.rs:handle_movement):**
```rust
// logic.rs
if let Err(e) = attempt_semantic_walk(&mut state, trigger) {
    // lookup failed
    let dynamic_room = create_dynamic_room(trigger, "A place you have never seen before.");
    // creates pseudo-room for invalid destinations
}

// If no match: creates dynamic pseudo-room
```

**Impact:** The doc implies that invalid destinations would fail. The actual code creates a dynamic pseudo-room. This is a **feature**, not a bug, but it's undocumented.

**Fix:** Add to `docs/system/navigation.md`:
> "If the destination is not found in the map, the engine creates a dynamic pseudo-room with a generic description. This allows the narrative to describe journeys to unknown places."

#### Discrepancy #7: create_dynamic_room Not Documented (MEDIUM)

**Actual:** `create_dynamic_room()` in `engine/logic.rs` is the factory for dynamic pseudo-rooms.

**Impact:** An AI modifying navigation logic wouldn't know about this function's existence or behavior.

---

## 4. Summary of All Discrepancies

### By Severity

| Severity | Count | Issues |
|----------|-------|--------|
| HIGH | 3 | game_flow Phase 4.5 order, system.md server_helpers missing, system.md duplicate |
| MEDIUM | 8 | game_flow Phase 5 location, retry flow imprecise, 8-layer vs 7-layer, navigation 6 items |
| LOW | 5 | navigation minor items |

### By Document

| Document | Discrepancies | HIGH | MEDIUM | LOW |
|----------|--------------|------|--------|-----|
| `docs/system/game_flow.md` | 6 | 2 | 3 | 1 |
| `docs/architecture/system.md` | 2 | 1 | 0 | 1 |
| `docs/system/navigation.md` | 8 | 0 | 5 | 3 |

---

## 5. Recommendations

### Immediate (High Severity)

1. **Fix system.md: Remove server_helpers module** (lines 204-205)
   - The module doesn't exist; functions are in `fixtures.rs` and `context.rs`
   - Update to: `create_app_for_testing` and `create_app_for_testing_with_settings` live in `test_support/fixtures.rs`

2. **Fix system.md: Remove duplicate test_app_builder entry** (line 205)
   - Remove second occurrence

3. **Update game_flow.md: Clarify Phase 4.5 movement processing**
   - Document that `handle_movement` is called DURING the quantifier result processing, not as a separate step
   - Clarify that movement is applied after quantifier result, before trigger eval

### Medium-term

4. **Update game_flow.md: Add "What happens if no room match"**
   - Document dynamic room creation fallback

5. **Update navigation.md: Add dynamic room creation section**
   - Document `create_dynamic_room` behavior

6. **Update game_flow.md: Clarify "7-layer" is actually 8 layers**
   - Add note: "Layer 0-7 + safety margin (Phi)"

7. **Add a "Doc-Code Consistency Checklist"** to the build process
   - Run a check that documented modules exist in src/
   - Flag discrepancies before documentation becomes stale

---

## 6. Priority Fix List

| Priority | Issue | File | Fix |
|----------|--------|------|-----|
| 1 | server_helpers module missing | `docs/architecture/system.md` | Remove from docs |
| 2 | Duplicate test_app_builder | `docs/architecture/system.md` | Remove duplicate |
| 3 | Phase 4.5 order misleading | `docs/system/game_flow.md` | Clarify movement processing |
| 4 | Dynamic room not documented | `docs/system/navigation.md` | Add section |
| 5 | 7-layer vs 8-layer | `docs/system/game_flow.md` | Fix to 8-layer |
| 6 | Quantifier produces NpcEvent | `docs/system/game_flow.md` | Clarify engine computes events |

---

*Phase 4 complete. Proceeding to Phase 5: Module Mental Models.*