# Implementation Plan: Fix Diagnostic Signal Quality for All 12 Scenarios

**Date:** 2026-05-09
**Based on:** Diagnostic Baseline Report (overall score: 3.1/10)
**Goal:** Raise the overall diagnostic benchmark score from 3.1 to ≥ 6.0

---

## Overview

The 12 benchmark scenarios reveal **5 root causes** that can be fixed with **7 targeted code changes**. Each fix is scoped, testable, and improves multiple scenarios at once.

### Root Cause Map

| Root Cause | Scenarios Affected | Fix Location |
|-----------|-------------------|--------------|
| `map_llm_error` collapses structured errors | `llm_http_401`, `llm_http_429`, `llm_network_error`, `llm_parse_error` | `src/application/game_service/helpers.rs:90` |
| Empty LLM response silently accepted | `llm_empty_response` | `src/application/game_service/actions.rs:224-227` |
| Quantifier failures are silent | `quantifier_complete_failure`, `quantifier_low_confidence` | `src/application/game_service/actions.rs:232-247` + `src/narrative/agents/quantifier/core.rs` |
| Dynamic rooms created without signal | `dynamic_room_creation` | `src/engine/action_processing.rs:57-66` |
| Debug endpoint too shallow | ALL scenarios (state_visibility dimension) | `src/server/debug.rs` |
| Trigger skips invisible | `trigger_wrong_room_id` | `src/engine/trigger_eval.rs` |
| State reset inconsistent | `state_stuck_generating` | `src/application/game_service/actions.rs:44-52` |

---

## Architecture Decisions

1. **Preserve structured errors at the UI boundary.** The recent `LlmFailure` enum migration was correct. The fix is to stop collapsing it in `map_llm_error`.
2. **Add system-visible signals, not just logs.** A WARN log line is invisible to users. We will add `System` log entries and expand `DebugStateResponse`.
3. **Fail soft but visible.** Dynamic room creation and quantifier fallback are valid fallback behaviors, but they must leave an auditable trace.
4. **No breaking changes to public APIs.** The `GameService` trait and `DebugStateResponse` will be extended, not changed.

---

## Task List

### Task 1: Preserve Structured Error Detail in `map_llm_error`

**Description:**
Modify `map_llm_error` to include specific diagnostic details from `LlmFailure` variants instead of collapsing them to generic strings. HTTP status codes, URLs, and parse error hints must be visible in the user-facing error message.

**Files touched:**
- `src/application/game_service/helpers.rs` (line 90)

**Acceptance criteria:**
- [ ] `llm_http_401` error message contains `"401"`
- [ ] `llm_http_429` error message contains `"429"`
- [ ] `llm_network_error` error message contains the URL or detail
- [ ] `llm_parse_error` error message contains `"parse"` or `"format"`
- [ ] `narrative_generation_failure` still works (regression test)
- [ ] All existing tests pass

**Verification:**
- [ ] Run `python scripts/diagnostic_benchmark.py`
- [ ] Confirm `llm_http_401` score improved from 1.7 to ≥ 6.0
- [ ] Confirm `llm_http_429` score improved from 1.7 to ≥ 6.0
- [ ] Confirm `llm_network_error` score improved from 2.3 to ≥ 5.0

**Estimated scope:** Small (1 function, ~15 lines)
**Dependencies:** None

---

### Task 2: Treat Empty LLM Response as an Error

**Description:**
Currently `MockBackend::with_empty_response()` returns `Ok("")`, and the engine accepts it. In production, an empty response from the LLM means something went wrong (model unloaded, context truncated, etc.). Treat empty narration text as an error and set `GenerationStatus::Error`.

**Files touched:**
- `src/application/game_service/actions.rs` (lines 224-227 — after `backend.narrate_action`)

**Acceptance criteria:**
- [ ] Empty narration response sets `GenerationStatus::Error("LLM Error: empty response")`
- [ ] Empty response does NOT get added to narration history
- [ ] `llm_empty_response` scenario score improves

**Verification:**
- [ ] Run benchmark, confirm `llm_empty_response` score ≥ 6.0

**Estimated scope:** XS (1 conditional, ~5 lines)
**Dependencies:** None

---

### Task 3: Surface Quantifier Failures to UI

**Description:**
When the quantifier fails completely or returns low confidence, the engine silently falls back to static NPCs. Add a visible `System` log entry so the user (and debugger) knows quantification was uncertain or failed.

**Files touched:**
- `src/narrative/agents/quantifier/core.rs` (fallback path)
- `src/narrative/agents/quantifier/core.rs` (low confidence path)
- `src/application/game_service/actions.rs` (lines 232-247 — quantifier result handling)

**Acceptance criteria:**
- [ ] Quantifier complete failure adds a `System` log: `"[System] NPC detection failed — using room defaults"`
- [ ] Quantifier low confidence adds a `System` log: `"[System] NPC detection uncertain — using room defaults"`
- [ ] `GenerationStatus` remains `Idle` (not an error — this is graceful degradation)
- [ ] The system log is visible in the UI story log

**Verification:**
- [ ] Run benchmark, confirm `quantifier_complete_failure` score ≥ 4.0
- [ ] Run benchmark, confirm `quantifier_low_confidence` score ≥ 4.0

**Estimated scope:** Small (2-3 files, ~10 lines)
**Dependencies:** None

---

### Task 4: Make Dynamic Room Creation Visible

**Description:**
When `attempt_semantic_walk` fails and a dynamic room is created, add a `System` log entry. Also add the dynamic room ID to the debug endpoint response.

**Files touched:**
- `src/engine/action_processing.rs` (lines 57-66 — `handle_movement`)
- `src/server/debug.rs` (add `dynamic_rooms` field)
- `src/server/debug.rs` (add `dynamic_room_count` field)

**Acceptance criteria:**
- [ ] Dynamic room creation adds a `System` log: `"[System] Entered unknown location: dynamic_<name>"`
- [ ] `/debug/state` includes `dynamic_rooms: Vec<String>`
- [ ] `/debug/state` includes `dynamic_room_count: usize`

**Verification:**
- [ ] Run benchmark, confirm `dynamic_room_creation` score ≥ 6.0

**Estimated scope:** Small (2 files, ~15 lines)
**Dependencies:** None

---

### Task 5: Expand `/debug/state` with Pipeline Context

**Description:**
The debug endpoint currently returns only the last 5 log entries and basic state. Expand it to include diagnostic fields that help debug ANY failure scenario.

**Files touched:**
- `src/server/debug.rs` (expand `DebugStateResponse`)
- `src/server/debug.rs` (expand `debug_state_handler`)

**New fields to add:**
- `last_error: Option<String>` — the most recent error message
- `error_history: Vec<String>` — last N errors (not just logs)
- `backend_name: String` — which LLM backend was used
- `quantifier_confidence: Option<String>` — High/Medium/Low
- `last_prompt_length: Option<usize>` — length of last prompt sent
- `last_response_length: Option<usize>` — length of last response
- `dynamic_rooms: Vec<String>` — list of dynamic room IDs

**Acceptance criteria:**
- [ ] All new fields compile and serialize correctly
- [ ] `/debug/state` returns valid JSON with new fields
- [ ] No panic when state is locked
- [ ] Existing tests still pass

**Verification:**
- [ ] Run benchmark — `state_visibility` scores should improve across ALL scenarios
- [ ] Manual curl test: `curl http://localhost:3000/debug/state | jq .`

**Estimated scope:** Medium (1 file, ~30 lines + struct changes)
**Dependencies:** Task 4 (for `dynamic_rooms` field)

---

### Task 6: Add Trigger Skip Logging

**Description:**
When a trigger is evaluated but skipped (e.g., wrong room_id, already fired, condition not met), log a debug or system message. This makes trigger misconfiguration visible.

**Files touched:**
- `src/engine/trigger_eval.rs` (evaluate trigger logic)

**Acceptance criteria:**
- [ ] Trigger skipped due to wrong room logs: `"[Debug] Trigger '<name>' skipped: room_id mismatch (expected X, current Y)"`
- [ ] Trigger skipped due to already fired logs: `"[Debug] Trigger '<name>' skipped: already fired"`
- [ ] Trigger skipped due to condition logs: `"[Debug] Trigger '<name>' skipped: condition not met"`

**Verification:**
- [ ] Run benchmark, confirm `trigger_wrong_room_id` score ≥ 4.0

**Estimated scope:** Small (1 file, ~10 lines)
**Dependencies:** None

---

### Task 7: Ensure Consistent State Reset on All Error Paths

**Description:**
In `execute_action`, the `FreeAction` path has multiple early returns (narration error, quantifier lock failure, trigger error). Some paths call `reset_generating`, some call `set_error_and_reset`, and some just return. Ensure every exit path leaves `generation_state` in a deterministic state.

**Files touched:**
- `src/application/game_service/actions.rs` (lines 44-52 — `execute_action` FreeAction path)

**Acceptance criteria:**
- [ ] All early returns from `FreeAction` path reset `generation_state` to `Idle` or `Error`
- [ ] No path leaves `generation_state.status == Generating` after `execute_action` returns
- [ ] Trigger narration failure sets `Error` status (currently it does, but verify)
- [ ] `state_stuck_generating` benchmark passes with score ≥ 6.0

**Verification:**
- [ ] Run benchmark, confirm `state_stuck_generating` score ≥ 6.0
- [ ] Add a dedicated unit test: call `execute_action` with failing backend, assert status is Error or Idle

**Estimated scope:** Medium (1 file, ~20 lines — careful refactoring)
**Dependencies:** None (but should be done after Tasks 1-3 to avoid conflicts)

---

### Task 8: Update Benchmark Tests for New Expected Behavior

**Description:**
After implementing fixes, the benchmark tests will need updated assertions and possibly revised scoring logic to reflect the new expected behavior.

**Files touched:**
- `tests/diagnostic_benchmark.rs` (all 12 scenarios)

**Acceptance criteria:**
- [ ] All 12 scenarios have updated assertions matching the new behavior
- [ ] Scoring logic reflects the improved signals
- [ ] Benchmark compiles without clippy warnings

**Verification:**
- [ ] `cargo nextest run --test diagnostic_benchmark --no-capture` passes
- [ ] `python scripts/diagnostic_benchmark.py` shows overall score ≥ 6.0

**Estimated scope:** Medium (1 file, ~30 lines of changes across 12 functions)
**Dependencies:** Tasks 1-7

---

## Checkpoint: After Tasks 1-7

Before updating the benchmark scores (Task 8), verify:

- [ ] `cargo nextest run` passes (all existing tests)
- [ ] `python build.py` passes (full validation)
- [ ] `python scripts/diagnostic_benchmark.py` runs without errors
- [ ] Overall benchmark score is ≥ 6.0

If the score is below 6.0, identify which scenarios are still underperforming and add targeted fixes before finalizing.

---

## Execution Order

```
Task 1 (map_llm_error) ─┐
Task 2 (empty response) ─┼─► Checkpoint A: LLM scenarios improved
Task 3 (quantifier) ────┘

Task 4 (dynamic rooms) ──► Checkpoint B: Navigation scenarios improved

Task 6 (trigger logging) ──► Checkpoint C: Trigger scenarios improved

Task 7 (state reset) ──► Checkpoint D: State management improved

Task 5 (debug endpoint) ──► Checkpoint E: State visibility improved across all

Task 8 (benchmark updates) ──► Final verification
```

**Why this order:**
- Tasks 1-3 are independent and touch different files in `application/game_service/` — do them together to avoid merge conflicts
- Task 4 is independent
- Task 7 should come after Tasks 1-3 because it touches the same `FreeAction` path in `application/game_service/actions.rs`
- Task 5 (debug endpoint) can happen anytime but should be before Task 8
- Task 8 is always last

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `map_llm_error` changes break existing tests that assert exact error strings | Medium | Update test assertions to match new format; keep messages semantically similar |
| Debug endpoint fields expose too much data | Low | All fields are internal state only; endpoint is dev-only |
| Quantifier system log spam | Low | Only log on failure/low-confidence, not on every request |
| State reset refactoring introduces race condition | Medium | Add unit test covering all error paths; run benchmark 3x to check flakiness |

---

## Open Questions

1. **Should quantifier low confidence be an Error or just a System log?** 
   - Recommendation: System log only. Low confidence is not a failure — it's uncertainty. The game should continue.
   
2. **Should dynamic room creation be an Error or just a System log?**
   - Recommendation: System log only. Dynamic rooms are a valid game mechanic for unknown destinations. But the player should be told they're somewhere strange.

3. **How many errors should `/debug/state` return?**
   - Recommendation: Last 10 errors, or all errors from the current session (whichever is smaller).

---

## Expected Score Improvements

| Scenario | Current | Expected After Fixes | Delta |
|----------|---------|---------------------|-------|
| `llm_http_401` | 1.7 | 7.5 | +5.8 |
| `llm_http_429` | 1.7 | 7.5 | +5.8 |
| `llm_network_error` | 2.3 | 6.5 | +4.2 |
| `llm_parse_error` | 4.3 | 7.0 | +2.7 |
| `llm_empty_response` | 2.3 | 7.0 | +4.7 |
| `llm_timeout` | 5.7 | 7.0 | +1.3 |
| `quantifier_complete_failure` | 1.7 | 4.5 | +2.8 |
| `quantifier_low_confidence` | 1.7 | 4.5 | +2.8 |
| `dynamic_room_creation` | 4.7 | 7.0 | +2.3 |
| `trigger_wrong_room_id` | 2.3 | 5.0 | +2.7 |
| `state_stuck_generating` | 4.0 | 7.0 | +3.0 |
| `narrative_generation_failure` | 4.3 | 6.5 | +2.2 |
| **OVERALL** | **3.1** | **6.3** | **+3.2** |

*Note: These are estimates. Actual scores depend on exact implementation details.*
