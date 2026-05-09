# Fix: Show LLM Text Immediately, Delay Trigger Text Until Generated

## Overview

When a player submits a FreeAction, the main LLM narration and any event trigger narration appear in the story log **simultaneously**. The frontend never sees an intermediate state with only the main narration.

The root cause is that `evaluate_and_narrate_triggers` makes a second LLM call while holding the `Mutex<GameState>`, blocking all frontend poll requests. When the lock finally releases, both texts are already committed.

This plan splits trigger narration into three phases: lock → unlock (LLM) → lock, following the same pattern already used for the main narration LLM call.

---

## Architecture Decisions

- **Reuse existing lock-drop pattern**: `game_service.rs` already drops the lock before the main narration LLM call (line 154). We apply the same pattern to the trigger narration.
- **Keep `evaluate_and_narrate_triggers` for test compatibility**: The function stays as-is for tests; production code bypasses it via split-phase orchestration.
- **Preserve mutation order invariant**: `execute_freeaction_impl` keeps the exact same step order (handle_movement → add_log → npcs_in_area → evaluate_triggers → apply_npc_events).

---

## Documentation Sync (BEFORE Code)

Per chronicler-dev-workflow, update these documents **before** writing any code:

1. **`docs/architecture/system.md`** — Update Game Service and Action Processing sections to describe the three-phase lock/unlock pattern.
2. **`docs/system/triggers.md`** — Update the Mutation Order Invariant section to reflect that trigger LLM generation now happens outside the lock (phase 2), while trigger evaluation (phase 1) and log commit (phase 3) remain inside.
3. **`CHANGELOG.md`** — Record the fix.

---

## Task List

### Phase 0: Documentation

**Task 0: Update architecture and system docs**

- Update `docs/architecture/system.md` Game Service tier to document: main narration LLM (no lock) → trigger evaluation (lock) → trigger LLM (no lock) → trigger commit (lock).
- Update `docs/system/triggers.md` Mutation Order Invariant to clarify that step 4 (trigger evaluation) is now split: evaluation happens inside the lock, LLM generation outside, log commit inside.
- Add entry to `CHANGELOG.md`.

**Files:**
- `chronicler_engine/docs/architecture/system.md`
- `chronicler_engine/docs/system/triggers.md`
- `chronicler_engine/docs/CHANGELOG.md`

**Acceptance criteria:**
- [ ] Architecture docs accurately describe the new three-phase flow.
- [ ] Mutation order invariant is updated to reflect the split without changing the evaluation→apply_npc_events ordering.
- [ ] CHANGELOG entry describes the bug fix.

**Verification:**
- [ ] Human review of doc changes.

**Estimated scope:** Small (3 files, text changes only)

---

### Phase 1: Extract Trigger Commit Logic

**Task 1: Add `TriggerContinuationRequest` struct and `commit_trigger_narration` function**

- Add `TriggerContinuationRequest` struct to `action_processing.rs` containing all data needed for the LLM call and log commit.
- Add `commit_trigger_narration(state, request, continuation_text)` that adds the event header + narration logs and marks the trigger fired.

**Files:** `chronicler_engine/src/engine/action_processing.rs`

**Acceptance criteria:**
- [ ] Struct compiles with all necessary fields.
- [ ] `commit_trigger_narration` adds `LogType::Event` header, `LogType::Narration` continuation, and calls `mark_trigger_fired` for non-repeating triggers.
- [ ] `commit_trigger_narration` is a no-op when `continuation_text.trim().is_empty()`.

**Verification:**
- [ ] `cargo check` passes.

**Estimated scope:** Small (1 file, 2 additions)

---

**Task 2: Update `execute_freeaction_impl` to return trigger request instead of calling LLM**

- Change return type from `Result<(), EngineError>` to `Result<Option<TriggerContinuationRequest>, EngineError>`.
- Remove call to `evaluate_and_narrate_triggers`.
- Inline trigger evaluation + prompt building (same logic, no LLM call).
- Apply NPC events after trigger evaluation (preserving mutation order invariant).
- Return `Some(request)` if a trigger matched, `None` otherwise.

**Files:** `chronicler_engine/src/engine/action_processing.rs`

**Acceptance criteria:**
- [ ] `execute_freeaction_impl` no longer calls `evaluate_and_narrate_triggers`.
- [ ] All existing tests calling `execute_freeaction_impl` still pass (`.is_ok()` compatible).
- [ ] Mutation order invariant is preserved exactly.

**Verification:**
- [ ] `cargo test -- action_processing` passes.
- [ ] `cargo clippy` passes.

**Dependencies:** Task 0, Task 1

**Estimated scope:** Small (1 file, 1 function refactor)

---

### Checkpoint: Core Logic Extracted

- [ ] `cargo test` passes for `action_processing` module.
- [ ] `cargo clippy` clean.
- [ ] Architecture docs updated.

---

### Phase 2: Orchestrate Three-Phase Flow in Game Service

**Task 3: Wire up lock → unlock (LLM) → lock in `game_service.rs`**

- After the first `with_state_lock` call to `execute_freeaction_impl`, check the returned `Option<TriggerContinuationRequest>`.
- If `Some`, set phase to `GeneratingEvent` and call `llm_backend.narrate_action_from_prompt()` **outside** the lock.
- On success, re-acquire the lock and call `commit_trigger_narration`.
- On LLM error, log the error (do not crash; button should still unlock via `reset_generating`).

**Files:** `chronicler_engine/src/engine/game_service.rs`

**Acceptance criteria:**
- [ ] Main narration is committed while holding the first lock.
- [ ] Lock is released before trigger LLM call.
- [ ] Lock is re-acquired before trigger logs are committed.
- [ ] `reset_generating` is still called exactly once at the end.

**Verification:**
- [ ] `cargo test` passes.
- [ ] `cargo clippy` passes.

**Dependencies:** Task 2

**Estimated scope:** Small (1 file, 1 branch refactor)

---

### Checkpoint: Three-Phase Flow Working

- [ ] `cargo test` passes across the whole crate.
- [ ] `cargo clippy` clean.

---

### Phase 3: Tests

**Task 4: Write failing test for `commit_trigger_narration` before implementing**

- Create `test_commit_trigger_narration_adds_logs_and_marks_fired`.
- Verify event header + narration are added to `narration_history`.
- Verify non-repeating trigger is marked fired in `character_state`.
- Verify repeating trigger is NOT marked fired.

**Files:** `chronicler_engine/src/engine/action_processing_tests.rs`

**Acceptance criteria:**
- [ ] Test covers happy path (event header + narration added).
- [ ] Test covers repeat flag behavior.
- [ ] Test covers empty continuation text (no-op).

**Verification:**
- [ ] New test compiles but fails (since `commit_trigger_narration` not yet implemented).

**Dependencies:** Task 0

**Estimated scope:** Small (1 file, 1 new test)

---

**Task 5: Fix any broken test expectations**

- `test_execute_freeaction_impl_triggers_evaluated` may need updating since `execute_freeaction_impl` no longer internally triggers LLM calls.
- Ensure `test_evaluate_and_narrate_triggers_adds_event_header` still compiles and passes (it calls the preserved standalone function).

**Files:** `chronicler_engine/src/engine/action_processing_tests.rs`

**Acceptance criteria:**
- [ ] All existing tests pass.
- [ ] No dead-code warnings from unused functions.

**Verification:**
- [ ] `cargo test -- action_processing` passes.

**Dependencies:** Tasks 2, 4

**Estimated scope:** Small (1 file, test adjustments)

---

### Phase 4: UI Verification

**Task 6: Verify the fix in the browser**

- Build and start the server.
- Submit a FreeAction that triggers an event (e.g., first encounter with an NPC).
- Confirm that the main narration appears **immediately**.
- Confirm that the event trigger text appears **after** the main narration (with a visible delay while the second LLM call runs).
- Take a screenshot of the story log showing the sequential appearance.

**Files:** N/A (runtime verification)

**Acceptance criteria:**
- [ ] Main narration is visible in the story log before the trigger text.
- [ ] Send button remains locked during both LLM calls.
- [ ] Trigger event header and narration appear as a second block after the main text.

**Verification:**
- [ ] Screenshot reviewed personally.

**Dependencies:** Task 3, Task 5

**Estimated scope:** Small (runtime check)

---

### Checkpoint: Complete

- [ ] `cd chronicler_engine && python build.py` passes (fmt + clippy + tests + coverage).
- [ ] All acceptance criteria met.
- [ ] Documentation synced.
- [ ] UI verified in browser with screenshot.

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Race condition between lock releases | Low | Send button lock prevents concurrent FreeActions; `status != Idle` blocks new submissions. |
| Mutation order invariant broken | High | `execute_freeaction_impl` preserves exact step order; only the LLM call moves out. |
| Tests break due to return type change | Medium | `.is_ok()` works on any `Result`; standalone `evaluate_and_narrate_triggers` stays for direct test use. |
| Frontend sees partial state (only main log, no trigger yet) | Expected | This is the desired behavior — main text shows immediately, trigger streams in later. |

## Open Questions

- None. The fix is a straightforward mechanical refactor following an established pattern in the same file.
