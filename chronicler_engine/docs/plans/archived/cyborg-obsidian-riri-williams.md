# Implementation Plan: Restrict Message Deletion & Inline Location/Event Headers

## Overview

Execute the approved Option A approach: move location and event headers into optional metadata fields on `LogEntry`. Only the last message in history can be deleted. Headers are consumed as "pending" state by `add_log` and render inside the narration div.

## Architecture Decisions

- `location_header` and `event_header` are `Option<String>` on `LogEntry`; only populated on the entry immediately after the triggering state change.
- `pending_location` and `pending_event` are `Option<String>` on `NarrativeState`; they act as a one-shot mailbox for `add_log`.
- `delete_log(id: u64)` is replaced by `delete_last_log()` which pops the final `LogEntry` from `history`.
- Legacy snapshots are migrated in-memory on load by merging orphan location/event entries into the following entry.

---

## Task List

### Phase 1: Data Model & Core State

#### Task 1: Extend `LogEntry`, `NarrativeState`, and `GameStateSnapshot`

**Description:** Add `location_header`, `event_header`, `pending_location`, and `pending_event` fields. Update `GameStateSnapshot` with `#[serde(default)]` for backward compatibility.

**Acceptance criteria:**
- [ ] `LogEntry` compiles with new optional fields.
- [ ] `NarrativeState` compiles with new optional fields and still derives `Default`.
- [ ] `GameStateSnapshot` deserializes old JSON safely (new fields default to `None`).

**Verification:**
- [ ] `cargo check` passes for `src/model/`.

**Dependencies:** None

**Files likely touched:**
- `src/model/state.rs`
- `src/model/state_snapshot.rs`

**Estimated scope:** Small

---

#### Task 2: Update `add_log` and implement `delete_last_log`

**Description:** `add_log` checks `pending_location` and `pending_event`, moves them into the new `LogEntry`, then clears the flags. Replace `delete_log(id)` with `delete_last_log()` that pops the last history item.

**Acceptance criteria:**
- [ ] `add_log` consumes `pending_location` into `location_header` and `pending_event` into `event_header`.
- [ ] `delete_last_log` removes the last entry and returns `Err` if history is empty.
- [ ] `edit_log(id)` remains unchanged.

**Verification:**
- [ ] Unit tests in `src/model/state_tests.rs` pass.

**Dependencies:** Task 1

**Files likely touched:**
- `src/model/state.rs`
- `src/model/state_tests.rs`

**Estimated scope:** Small

---

#### Task 3: Simplify `is_last_ai_response_event_continuation`

**Description:** Replace the backward scan for `LogType::Event` with a direct check of `event_header` on the last AI response entry.

**Acceptance criteria:**
- [ ] Method returns `true` iff the last narration/dialogue entry has `event_header.is_some()`.
- [ ] Existing retry behaviour is preserved.

**Verification:**
- [ ] `cargo nextest run` passes for `src/model/state_tests.rs`.

**Dependencies:** Task 1

**Files likely touched:**
- `src/model/state.rs`

**Estimated scope:** XS

---

### Checkpoint: After Tasks 1-3
- [ ] `cargo check` clean for `src/model/`.
- [ ] All `state_tests.rs` tests pass.

---

### Phase 2: Engine Logic

#### Task 4: Update `handle_movement`

**Description:** Remove the `add_log` call that creates the standalone location entry. Instead, set `state.narrative.pending_location = Some(current_room.name.clone())`.

**Acceptance criteria:**
- [ ] `handle_movement` no longer calls `add_log` for location headers.
- [ ] After a successful movement, `state.narrative.pending_location` is `Some(room_name)`.

**Verification:**
- [ ] `cargo nextest run` passes for `src/engine/action_processing_tests.rs` after test updates (Task 11).

**Dependencies:** Task 1

**Files likely touched:**
- `src/engine/action_processing.rs`

**Estimated scope:** Small

---

#### Task 5: Update trigger narration commit

**Description:** In `commit_trigger_narration` and `evaluate_and_narrate_triggers`, remove the separate `LogType::Event` `add_log` call. Set `pending_event` on the narrative state before adding the continuation narration.

**Acceptance criteria:**
- [ ] `commit_trigger_narration` sets `pending_event` instead of adding an `Event` log.
- [ ] `evaluate_and_narrate_triggers` does the same.
- [ ] Non-repeat triggers still call `mark_trigger_fired`.

**Verification:**
- [ ] `cargo nextest run` passes for `src/engine/action_processing_tests.rs` after test updates (Task 11).

**Dependencies:** Task 1

**Files likely touched:**
- `src/engine/action_processing.rs`

**Estimated scope:** Small

---

### Checkpoint: After Tasks 4-5
- [ ] `cargo check` clean for `src/engine/`.

---

### Phase 3: Rendering & HTTP

#### Task 6: Update templates and `LogEntryView`

**Description:** Remove `is_location`/`is_event` heuristics. Add `location_header` and `event_header` to `LogEntryView`. Update the Askama `StoryLogTemplate` to render headers inside the narration div.

**Acceptance criteria:**
- [ ] `LogEntryView::from` reads `location_header` and `event_header` directly.
- [ ] Template renders location header markup inside the same div when `location_header` is present.
- [ ] Template renders event header markup inside the same div when `event_header` is present.
- [ ] Legacy `is_location`/`is_event` logic is removed.

**Verification:**
- [ ] `cargo nextest run` passes for `src/server/templates_tests.rs` after test updates (Task 11).

**Dependencies:** Task 1

**Files likely touched:**
- `src/server/templates.rs`
- `src/server/templates_tests.rs`

**Estimated scope:** Medium

---

#### Task 7: Update delete HTTP endpoint

**Description:** Replace `delete_history_handler` with a parameterless `POST /history/delete` that calls `delete_last_log`. Return 400 (Bad Request) when history is empty. Update the router.

**Acceptance criteria:**
- [ ] Route changes from `POST /history/:id/delete` to `POST /history/delete`.
- [ ] Handler calls `delete_last_log()` and saves the snapshot on success.
- [ ] Returns 400 when history is empty.

**Verification:**
- [ ] `cargo nextest run` passes for `tests/components/fragment.rs` after test updates (Task 11).

**Dependencies:** Task 2

**Files likely touched:**
- `src/server/fragments/history.rs`
- `src/server/mod.rs`
- `tests/components/fragment.rs`

**Estimated scope:** Small

---

#### Task 8: Update frontend JavaScript

**Description:** Update `deleteMessage()` in `assets/index.html` to call the new parameterless endpoint. Remove the `id` argument and related DOM logic.

**Acceptance criteria:**
- [ ] `deleteMessage()` calls `POST /history/delete` with no path parameter.
- [ ] No orphaned `id` references remain in the delete flow.

**Verification:**
- [ ] Manual visual check or grep confirms the change.

**Dependencies:** Task 7

**Files likely touched:**
- `assets/index.html`

**Estimated scope:** XS

---

### Checkpoint: After Tasks 6-8
- [ ] `cargo check` clean for `src/server/`.
- [ ] Frontend JS updated.

---

### Phase 4: Tests & Migration

#### Task 9: Update engine unit tests

**Description:** Rewrite assertions in `action_processing_tests.rs` that expect separate location/event entries. Expect single entries with `location_header` or `event_header` metadata instead.

**Acceptance criteria:**
- [ ] `test_commit_trigger_narration_adds_event_header_and_narration` asserts one entry with `event_header`.
- [ ] `test_evaluate_and_narrate_triggers` asserts one entry with `event_header`.
- [ ] Movement tests assert `pending_location` is set instead of a standalone log.

**Verification:**
- [ ] `cargo nextest run action_processing_tests` passes.

**Dependencies:** Tasks 4-5

**Files likely touched:**
- `src/engine/action_processing_tests.rs`

**Estimated scope:** Medium

---

#### Task 10: Update model and template unit tests

**Description:** Update `state_tests.rs` and `templates_tests.rs` for the new data model and rendering.

**Acceptance criteria:**
- [ ] `test_delete_log` is replaced with `test_delete_last_log`.
- [ ] Tests verify `add_log` absorbs `pending_location` and `pending_event`.
- [ ] Template tests verify headers render inside the narration div.

**Verification:**
- [ ] `cargo nextest run state_tests` passes.
- [ ] `cargo nextest run templates_tests` passes.

**Dependencies:** Tasks 2, 6

**Files likely touched:**
- `src/model/state_tests.rs`
- `src/server/templates_tests.rs`

**Estimated scope:** Medium

---

#### Task 11: Update component tests

**Description:** Update `tests/components/fragment.rs` to use the new delete endpoint and verify `delete_last_log` behaviour.

**Acceptance criteria:**
- [ ] `test_delete_history_handler_success` calls `/history/delete` and verifies last entry removal.
- [ ] `test_delete_history_handler_not_found` becomes `test_delete_history_handler_empty` and verifies 400 response.

**Verification:**
- [ ] `cargo nextest run --test components` passes.

**Dependencies:** Task 7

**Files likely touched:**
- `tests/components/fragment.rs`

**Estimated scope:** Small

---

#### Task 12: Implement legacy snapshot migration

**Description:** On snapshot load (`GameState::from_snapshot`), scan history for legacy standalone location/event entries and merge them into the following entry's metadata.

**Acceptance criteria:**
- [ ] A standalone location entry (`sender=Some(x)`, `text=""`, `Narration`) is merged into the next entry's `location_header` and removed.
- [ ] A standalone event entry (`log_type == Event`, `text=""`) is merged into the next entry's `event_header` and removed.
- [ ] Migration is idempotent: running twice on already-migrated data is a no-op.

**Verification:**
- [ ] Add a unit test that constructs legacy history, runs migration, and asserts the expected modern history.

**Dependencies:** Task 1

**Files likely touched:**
- `src/model/state.rs`
- `src/model/state_tests.rs`

**Estimated scope:** Medium

---

### Checkpoint: After Tasks 9-12
- [ ] `cargo nextest run` passes for the entire workspace.
- [ ] `cargo clippy` is clean.

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Askama template syntax errors | Medium | Compile after every template change; `cargo check` catches errors immediately. |
| Snapshot migration misses edge cases | Medium | Write a dedicated migration unit test with synthetic legacy data. |
| Retry logic breaks due to event detection change | High | Verify `is_last_ai_response_event_continuation` with a test that simulates a turn with a trigger. |

## Open Questions

- None—plan is approved.
