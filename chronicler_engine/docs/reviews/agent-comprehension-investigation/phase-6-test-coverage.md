# Phase 6: Critical Path Test Coverage

**Date:** 2026-05-30  
**Scope:** Verify critical invariants have test coverage  
**Method:** Subagent-based test file analysis + gap identification

---

## Executive Summary

All 7 critical invariants have at least **partial coverage**. However, 3 significant gaps exist:

1. **INV-003 (Swipe navigation)**: No integration test verifies state restoration end-to-end
2. **INV-005 (Mutation order)**: Tests verify outcome but not the handle_movement step
3. **INV-007 (Dynamic room creation)**: Only covered by diagnostic benchmark, not unit test

---

## 1. Behavior-to-Test Mapping

### INV-001: Main narration retry re-runs quantifier + triggers

**What it means:** When user retries main narration, the full pipeline re-runs: quantifier → movement → triggers.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `test_retry_main_narration_applies_new_quantifier_result` | `tests/flow_mock/retry_main.rs` | Retry applies new quantifier result |
| `test_main_retry_reevaluates_triggers` | `tests/flow_mock/retry_main.rs` | Retry re-evaluates triggers in new room |
| `test_retry_with_different_narration_text_reruns_quantifier` | `tests/flow_mock/retry_main.rs` | Different narration text triggers rerun |
| `test_double_retry_increments_swipe_and_reruns_quantifier` | `tests/flow_mock/retry_main.rs` | Multiple retries work correctly |

**Coverage:** ✅ FULL

**Gap:** None identified.

---

### INV-002: Event continuation retry preserves quantifier result

**What it means:** When user retries a trigger event, quantifier is NOT re-run — trigger continuation only.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `test_retry_event_continuation_preserves_quantifier_result` | `tests/flow_mock/retry_event.rs` | Quantifier NOT rerun on event retry |
| `test_event_retry_does_not_create_extra_swipe_on_narration` | `tests/flow_mock/retry_event.rs` | No extra swipe created |

**Coverage:** ✅ FULL

**Gap:** None identified.

---

### INV-003: Swipe navigation restores snapshot

**What it means:** When user navigates between swipes (left/right arrows), game state should restore from the swipe's snapshot_id.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `test_switch_swipe_generation_in_progress` | `tests/components/misc.rs` | HTTP 503 during generation |
| `test_switch_swipe_not_last_message` | `tests/components/misc.rs` | HTTP 400 for non-last message |
| `test_switch_swipe_missing_snapshot` | `tests/components/misc.rs` | HTTP 400 for missing snapshot_id |

**Coverage:** ⚠️ PARTIAL

**Gap:** ❌ No integration test verifies that switching to swipe N actually restores game state from snapshot_id N (movement, NPCs, narrative history all restored).

**What a complete test would verify:**
```rust
#[test]
fn test_swipe_navigation_restores_full_state() {
    // Arrange: Create state, navigate, create multiple swipes
    // Act: Switch to swipe 1 (earlier)
    // Assert:
    //   - movement.current_room_id restored
    //   - narrative.history restored (correct length)
    //   - npcs_in_area restored
    //   - active_swipe_index = 1
}
```

---

### INV-004: Trigger fires on TimesMet Eq 0 before increment

**What it means:** When a trigger has `condition: { TimesMet: ["Eq", 0] }`, it should fire when times_met is 0, BEFORE the increment happens.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `test_inv002_state_mutation_order` | `tests/invariant_contract_tests.rs` | trigger_match is_some when times_met=0, times_met becomes 1 after |
| `benchmark_trigger_wrong_room_id` | `tests/diagnostic/scenarios.rs` | TimesMet Eq 0 with wrong room → no fire |
| `benchmark_state_stuck_generating` | `tests/diagnostic/scenarios.rs` | TimesMet Eq 0 trigger fires then fails |

**Coverage:** ✅ FULL

**Gap:** None identified.

---

### INV-005: Mutation order handle_movement → add_log → evaluate_triggers → apply_npc_events

**What it means:** The four steps must happen in this order. Violation breaks triggers.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `test_inv002_state_mutation_order` | `tests/invariant_contract_tests.rs` | Verifies narration logged before trigger, trigger fires before times_met increment |

**Coverage:** ⚠️ PARTIAL

**Gap:** ❌ `handle_movement` ordering not explicitly verified. Tests verify outcome (trigger fires, times_met incremented after) but not that handle_movement runs first.

**What a complete test would verify:**
```rust
#[test]
fn test_mutation_order_complete_sequence() {
    // Arrange: Start in room A, trigger in room B with TimesMet Eq 0
    // Act: Execute free action "go to room B"
    // Assert each step:
    //   1. movement.current_room_id == "B" (handle_movement ran)
    //   2. narrative.history contains narration for "go to room B" (add_log ran)
    //   3. trigger_match is_some (evaluate_triggers ran)
    //   4. npc_encounter_log["B"].times_met == 1 (apply_npc_events ran AFTER trigger)
}
```

---

### INV-006: Cancellation on stale requests

**What it means:** If user cancels during generation, pipeline aborts gracefully without wasting LLM calls.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `test_inv004_cancellable_at_boundaries` | `tests/invariant_contract_tests.rs` | Pipeline returns Cancelled on token cancel |
| `test_pipeline_cancels_when_token_cancelled` | `tests/action_pipeline/pipeline.rs` | Status → Idle on cancel |
| `test_cancellation_resets_state_to_idle` | `tests/action_pipeline/pipeline.rs` | Status → Idle |
| `test_pipeline_cancels_after_main_narration` | `tests/action_pipeline/pipeline.rs` | Cancel during quantifier phase |

**Coverage:** ✅ FULL

**Gap:** None identified.

---

### INV-007: Dynamic room creation on invalid destination

**What it means:** When player tries to move to an unknown location, a dynamic pseudo-room is created instead of failing.

**Tests covering this:**

| Test | File | What It Covers |
|------|------|----------------|
| `benchmark_dynamic_room_creation` | `tests/diagnostic/scenarios.rs` | MisleadingMovementQuantifierBackend → non-existent room |

**Coverage:** ⚠️ PARTIAL (diagnostic benchmark only)

**Gap:** ❌ No unit regression test for `handle_movement` with invalid destination. The benchmark exists but no standard test.

**What a complete test would verify:**
```rust
#[test]
fn test_handle_movement_creates_dynamic_room_on_invalid_destination() {
    // Arrange: GameState with map (no "unknown_place" room)
    // Act: handle_movement(state, "unknown_place", &[])
    // Assert:
    //   - state.movement.current_room_id == dynamic_room.id
    //   - state.movement.dynamic_rooms contains the room
    //   - narrative.history contains "[System] Entered unknown location"
    //   - LogType is System
}
```

---

## 2. Gap Summary

| Invariant | Gap | Severity | Why It Matters |
|-----------|-----|----------|----------------|
| INV-003 | No integration test for state restoration | MEDIUM | Swipe navigation could silently fail to restore correct state |
| INV-005 | No explicit test for handle_movement step | MEDIUM | Refactoring could accidentally move handle_movement |
| INV-007 | No unit test for dynamic room creation | LOW | Only benchmark exists, could be skipped in CI |

---

## 3. Test File Organization

```
tests/
├── flow_mock/              ─── End-to-end flow with mock backends
│   ├── retry_main.rs       ─── INV-001 coverage
│   └── retry_event.rs      ─── INV-002 coverage
├── invariant_contract_tests.rs ─── INV-002, INV-004, INV-006
├── action_pipeline/
│   └── pipeline.rs         ─── INV-006 cancellation
├── components/
│   └── misc.rs             ─── INV-003 HTTP handlers (no state verification)
├── diagnostic/
│   └── scenarios.rs        ─── INV-003, INV-004, INV-007 benchmarks
└── logic_tests.rs          ─── Unit tests for engine logic
```

---

## 4. Recommendations

### Immediate (High Priority)

1. **Add INV-007 unit test** for `handle_movement` with invalid destination
   ```rust
   #[test]
   fn test_handle_movement_creates_dynamic_room_on_invalid_destination() {
       // Test the scenario covered only by benchmark
   }
   ```

2. **Add INV-003 integration test** for swipe state restoration
   ```rust
   #[test]
   fn test_swipe_navigation_restores_full_state() {
       // Verify movement, NPCs, history all restored
   }
   ```

### Medium-term (When Touching Related Code)

3. **Add INV-005 explicit mutation order test**
   ```rust
   #[test]
   fn test_mutation_order_complete_sequence() {
       // Explicitly verify each step in order
   }
   ```

4. **Add explicit violation tests** for mutation order
   ```rust
   #[test]
   #[should_panic(expected = "times_met must be 0 when triggers evaluate")]
   fn test_trigger_evaluation_violation_detected() {
       // Prove that violating the order is caught
   }
   ```

---

## 5. Coverage Summary

| Invariant | Covered | Gap | Severity |
|-----------|---------|-----|----------|
| INV-001: Main retry re-runs quantifier + triggers | ✅ FULL | None | - |
| INV-002: Event retry preserves quantifier | ✅ FULL | None | - |
| INV-003: Swipe restores snapshot | ⚠️ PARTIAL | No state verification test | MEDIUM |
| INV-004: TimesMet Eq 0 fires before increment | ✅ FULL | None | - |
| INV-005: Mutation order (complete) | ⚠️ PARTIAL | No handle_movement test | MEDIUM |
| INV-006: Cancellation on stale requests | ✅ FULL | None | - |
| INV-007: Dynamic room on invalid destination | ⚠️ PARTIAL | No unit test | LOW |

**Overall Coverage:** 7/7 invariants covered (some partial), 3 gaps identified.

---

*Phase 6 complete. All 6 phases of investigation finished.*