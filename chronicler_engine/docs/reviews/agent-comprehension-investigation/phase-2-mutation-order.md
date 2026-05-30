# Phase 2: State Mutation Order Invariant Analysis

**Date:** 2026-05-30  
**Scope:** Verify the trigger system's critical ordering constraint is discoverable and enforced  
**Method:** Line-by-line trace of `execute_freeaction_impl` + test coverage verification

---

## Executive Summary

The state mutation order invariant **is documented** in `docs/system/triggers.md` and **has test coverage**, but the invariant is **not enforced** — it relies on code structure. An AI modifying `execute_freeaction_impl` without reading the docs could silently break the trigger system.

**Critical Finding:** The documentation contains an incorrect claim about `LogType::Event` which should be `event_header` on `LogEntry`/`Message`/`Swipe`.

---

## 1. The Invariant

From `docs/system/triggers.md`:

> **Mutation Order Invariant**  
> Steps 4a and 4c happen in the application pipeline (`ActionPipeline`), not inside the engine function:

| Step | Operation | Location | Why It Must Come Here |
|------|-----------|----------|----------------------|
| 1 | `handle_movement()` — update `current_room_id` | `action_processing.rs:41` | Room must be current before NPCs are resolved |
| 2 | Resolve current NPCs from quantifier result | `action_processing.rs:142-149` | Uses updated `current_room_id` |
| 3 | `state.add_log(narration_text)` | `action_processing.rs:154` | Narration must be in history before triggers read it |
| 4a | `evaluate_triggers()` — **BEFORE** times_met increment | `action_processing.rs:159` | Reads history for context |
| 4b | Trigger LLM call (outside lock) | `application/pipeline.rs` | Frontend can poll main narration |
| 4c | `commit_trigger_narration()` | `action_processing.rs:103` | Add trigger logs and mark fired |
| 5 | `apply_npc_events()` — times_met increment | `action_processing.rs:172` | **AFTER** trigger evaluation |

### The Critical Timing Rule

> Triggers are evaluated **BEFORE** `times_met` is incremented.

If step 5 happens before step 4a, `TimesMet Eq 0` would immediately become false, and triggers would never fire.

---

## 2. Code Trace

### `execute_freeaction_impl` (`action_processing.rs:128-180`)

```rust
pub fn execute_freeaction_impl(
    state: &GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<TurnResult, EngineError> {
    // ─── Step 1: Capture previous NPCs ──────────────────────────────────────
    let previous_room_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();
    let previous_npc_ids: Vec<String> = previous_room_npcs.iter().map(|n| n.id.clone()).collect();

    // ─── Step 2: handle_movement() ───────────────────────────────────────────
    let mut next_state = handle_movement(
        state.clone(),
        ctx.quantifier_result.movement.destination.as_deref(),
        &ctx.quantifier_result.npcs.npc_ids,
    )?;

    // ─── Step 3: Resolve current NPCs ────────────────────────────────────────
    let current_npcs: Vec<NpcCard> = ctx
        .quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| next_state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids: Vec<String> = current_npcs.iter().map(|n| n.id.clone()).collect();

    // ─── Step 4: Log narration (before trigger eval) ─────────────────────────
    // [DOC: docs/system/triggers.md section: Mutation Order Invariant]
    // Order is load-bearing: narration logged first (step 1), then triggers evaluated
    // which read history for context (step 2), then NPC events applied (step 3).
    next_state.add_log(ctx.narration_text.to_string(), None, LogType::Narration);
    next_state.scene.npcs_in_area = current_npcs.clone();

    // ─── Step 5: Evaluate triggers (BEFORE times_met increment) ───────────────
    // Evaluate triggers BEFORE applying NPC events so that trigger conditions
    // (e.g., times_met) are checked against the pre-event state.
    let trigger_match = evaluate_triggers(&next_state)
        .into_iter()
        .next()
        .map(|(npc, trigger, idx)| TriggerMatch { ... });

    // ─── Step 6: Compute and apply NPC events ─────────────────────────────────
    let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
    next_state = apply_npc_events(next_state, &events.events)?;
    
    // ...
}
```

**Invariant preserved:** Step 5 (evaluate_triggers) happens before step 6 (apply_npc_events → times_met increment).

### `handle_movement` (`action_processing.rs:41-80`)

```rust
pub fn handle_movement(
    state: GameState,
    destination: Option<&str>,
    new_npc_ids: &[String],
) -> Result<GameState, EngineError> {
    // ...
    if previous_room_id != state.movement.current_room_id {
        for npc_id in new_npc_ids {
            set_currently_meeting(&mut state.npc_encounter_log, npc_id, true);
        }
    }
    // ...
}
```

**Note:** `set_currently_meeting` is called here for NPCs in the destination room, but `times_met` is NOT incremented here. The increment happens in `apply_npc_events` (step 6 in `execute_freeaction_impl`), which is AFTER trigger evaluation. ✅

### `apply_npc_events` (`action_processing.rs:83-99`)

```rust
pub fn apply_npc_events(state: GameState, events: &[NpcEvent]) -> Result<GameState, EngineError> {
    let mut state = state;
    for event in events {
        match event.event_type {
            NpcEventType::Entered => {
                set_currently_meeting(&mut state.npc_encounter_log, &event.npc_id, true);
                increment_times_met(&mut state.npc_encounter_log, &event.npc_id);  // ← HERE
            }
            NpcEventType::Left => {
                set_currently_meeting(&mut state.npc_encounter_log, &event.npc_id, false);
            }
        }
    }
    // ...
}
```

**Critical:** `increment_times_met` is called in `apply_npc_events`, which is called AFTER `evaluate_triggers` in `execute_freeaction_impl`. ✅

---

## 3. Test Coverage

### `tests/flow_mock/retry_event.rs`

| Test | What It Covers | Mutation Order Verified? |
|------|----------------|------------------------|
| `test_trigger_continuation_runs_quantifier_and_detects_new_npc` | Second quantifier detects NPCs from trigger text; times_met increments | ✅ Yes |
| `test_retry_main_narration_applies_new_quantifier_result` | Main retry re-runs quantifier + triggers | ✅ Yes |
| `test_main_retry_reevaluates_triggers` | Trigger fires again on main retry | ✅ Yes |
| `test_retry_event_continuation_preserves_quantifier_result` | Event retry preserves quantifier, re-runs trigger only | ✅ Yes |
| `test_retry_no_pre_main_snapshot` | Graceful failure when no snapshot | ✅ Yes |

### `tests/logic_tests.rs`

| Test | What It Covers |
|------|----------------|
| `test_times_met_increments_on_npc_enter` | Entered event increments times_met |
| `test_times_met_not_incremented_on_continued_presence` | Repeated presence doesn't increment |
| `test_currently_meeting_toggle_on_npc_enter_leave` | Entered/Left toggle currently_meeting |

### Gap Analysis

**Covered:**
- Trigger evaluation order relative to times_met increment ✅
- Main retry re-evaluates triggers ✅
- Event retry preserves quantifier result ✅
- Swipe navigation restores snapshot ✅

**Not covered:**
- What happens if narration is NOT logged before trigger eval (would triggers fail silently?)
- What happens if apply_npc_events is called BEFORE evaluate_triggers (would triggers never fire?)

**Recommendation:** Add explicit tests for violation scenarios to document that these orders are load-bearing.

---

## 4. Documentation vs Code Discrepancy

### Issue: `LogType::Event` Does Not Exist

From `docs/system/triggers.md`:

> "When a trigger fires, an event header with this name appears in the story log before the LLM-generated narration. There is no standalone `LogType::Event` entry."

This contradicts the earlier section of the same document:

> "Event headers have no edit or retry buttons. Are rendered with `.event-header` and `.event-timestamp` CSS classes"

The document says "no standalone `LogType::Event` entry" but the architecture description implies one exists.

**Actual implementation:**
- `LogType` enum (`model/state.rs:15-19`) has: Narration, Dialogue, System, Input — **NO Event variant**
- Trigger narrations are logged as `LogType::Narration`
- The `event_header: Option<String>` field on `LogEntry`/`Message`/`Swipe` marks them as trigger-based
- This is used for UI grouping and retry logic

**Impact:** An AI reading `docs/system/triggers.md` might look for `LogType::Event` and not find it, or might incorrectly implement one.

**Fix:** Update the documentation to explicitly state:
> "There is NO `LogType::Event`. Trigger narrations are `LogType::Narration` with `event_header` metadata. The `event_header` field distinguishes trigger-based narrations from regular narrations in the UI and retry logic."

---

## 5. Invariant Discoverability Assessment

### How Easy Is It to Discover?

| Source | Discoverability | Notes |
|--------|-----------------|-------|
| `docs/system/triggers.md` | HIGH | Explicit section titled "Mutation Order Invariant" with step-by-step table |
| `action_processing.rs` comments | HIGH | Line 151-153: `// [DOC: docs/system/triggers.md section: Mutation Order Invariant]` + explanation |
| Test comments | MEDIUM | Test names hint at behavior but don't explain why order matters |
| Code structure | LOW | Order is implicit in code; no runtime enforcement |

### How Enforced?

| Mechanism | Enforced? | Notes |
|-----------|-----------|-------|
| Code comments | YES (informational) | References doc, explains why order matters |
| Runtime checks | NO | `assert_state_consistency` checks room/NPC invariants, not mutation order |
| Test coverage | PARTIAL | Covered for happy path, not for violation scenarios |
| Compiler | NO | No type-level enforcement of order |

### Risk Assessment

| Scenario | Risk | Impact |
|----------|------|--------|
| AI adds new function that calls `apply_npc_events` before `evaluate_triggers` | HIGH | Triggers would silently stop firing |
| AI refactors `execute_freeaction_impl` to use different order | CRITICAL | Trigger system would break |
| AI adds new function that doesn't log narration before trigger eval | HIGH | Triggers would have no context |
| AI adds new trigger type that depends on different ordering | MEDIUM | Depends on specific trigger condition |

---

## 6. Recommendations

### Immediate

1. **Add explicit tests for mutation order violations**
   ```rust
   #[test]
   fn test_trigger_evaluation_must_happen_before_times_met_increment() {
       // Prove that if times_met increments before trigger eval,
       // TimesMet Eq 0 never fires
   }
   ```
   This documents the invariant as a test rather than relying on docs.

2. **Update `docs/system/triggers.md`**
   - Remove ambiguous references to `LogType::Event`
   - Add explicit statement: "There is NO `LogType::Event`. Trigger narrations are `LogType::Narration` with `event_header` metadata."
   - Add a "What Breaks If You Violate This Order" section with concrete examples

3. **Add doc comment to `execute_freeaction_impl`**
   ```rust
   /// Load-bearing mutation order:
   /// 1. handle_movement() → update room
   /// 2. resolve NPCs → get current NPCs
   /// 3. add_log() → narration in history
   /// 4. evaluate_triggers() → BEFORE times_met increment ← CRITICAL
   /// 5. apply_npc_events() → times_met increment ← AFTER trigger eval
   ```

### Medium-term

4. **Consider runtime enforcement** (if the invariant is critical enough)
   - Add a `phantom_mutation_token` field that is consumed and produced at each step
   - Type-level enforcement of order would be overkill but runtime assertions are possible
   - OR: Add a test that explicitly violates the order and asserts the wrong behavior fails

5. **Create "Key Invariants" document**
   - List all load-bearing invariants that an AI must preserve
   - Include violation scenarios and expected behavior
   - Place in `docs/architecture/` so it's in the default read path

---

## 7. Summary

| Finding | Severity | Status |
|---------|----------|--------|
| Mutation order invariant documented | HIGH | ✅ Present in `docs/system/triggers.md` |
| Invariant encoded in comments | HIGH | ✅ `action_processing.rs:151-158` |
| Invariant has test coverage | MEDIUM | ✅ Happy path covered |
| Invariant has violation tests | LOW | ❌ No explicit violation tests |
| `LogType::Event` does not exist | MEDIUM | ❌ Docs ambiguous, code correct |
| Code enforces invariant | LOW | ❌ Order is structural, not enforced |

**Overall Risk:** MEDIUM — The invariant is documented and has test coverage, but an AI could violate it without runtime checks. Adding violation tests would close this gap.

---

*Phase 2 complete. Proceeding to Phase 3: Tier Boundary Confusion Analysis.*