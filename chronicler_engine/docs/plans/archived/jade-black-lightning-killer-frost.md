# Plan: Extract MessageHistory from GameState

## Overview

Extract `Vec<Message>` and all operations on it from `GameState` / `NarrativeState` into a dedicated `MessageHistory` type in `src/model/message_history.rs`. This removes ~100 lines of message-management logic from `state.rs`, prevents callers from bypassing rules with direct `.push()`, and makes the message lifecycle independently testable.

The extraction **rejects** the original suggestion's name (`NarrativeLog`) and location (`engine/` or `application/`). `MessageHistory` lives in `model/` because `GameState` lives in `model/` and `arch-lint.toml` forbids `model/` from importing `engine/` or `application/`.

---

## Architecture Decisions

- **Name:** `MessageHistory` — the data is a chronological sequence of `Message` structs, not a "log".
- **Location:** `src/model/message_history.rs` — stays in the `model` layer.
- **Encapsulation:** `messages` field becomes private. Callers must use `MessageHistory` methods. No `messages_mut()` escape hatch.
- **Serde:** `MessageHistory` implements `Serialize` / `Deserialize` by transparently proxying to `Vec<Message>` so `NarrativeState` serialization does not change shape. `NarrativeSnapshot` already excludes messages, so snapshot format is unaffected.

---

## Task List

### Phase 1: Create MessageHistory

#### Task 1: Create `src/model/message_history.rs`

**Description:** Define `MessageHistory` and port all message operations from `GameState`. Move `MAX_LOG_ENTRIES` here.

**API:**
```rust
pub struct MessageHistory {
    messages: Vec<Message>,
}

impl MessageHistory {
    pub fn new() -> Self;
    pub fn from_messages(messages: Vec<Message>) -> Self;
    pub fn append(&mut self, message: Message);
    pub fn edit(&mut self, id: u64, new_text: String) -> crate::error::Result<()>;
    pub fn delete_last(&mut self) -> crate::error::Result<()>;
    pub fn get(&self, id: u64) -> Option<&Message>;
    pub fn last(&self) -> Option<&Message>;
    pub fn last_mut(&mut self) -> Option<&mut Message>;
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn iter(&self) -> impl Iterator<Item = &Message>;
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Message>;
    pub fn retain(&mut self, f: impl FnMut(&Message) -> bool);
    pub fn clear(&mut self);
    pub fn as_slice(&self) -> &[Message];
    pub fn replace(&mut self, messages: Vec<Message>);
    pub fn last_ai_response_index(&self) -> Option<usize>;
    pub fn last_input_index(&self) -> Option<usize>;
    pub fn last_input_text(&self) -> Option<(String, String)>;
    pub fn is_last_ai_response_event_continuation(&self) -> bool;
    pub fn to_log_entries(&self) -> Vec<LogEntry>;
}
```

**Note:** `push_message` logic (capacity cap at 1000, `pending_location`/`pending_event` absorption) **does not** move into `MessageHistory`. It stays in `GameState` because it depends on `NarrativeState`'s `pending_*` fields. `GameState` will build the `Message` and pass it to `history.append(message)`.

**Acceptance criteria:**
- [ ] `MessageHistory` defined with all methods above.
- [ ] `MAX_LOG_ENTRIES` lives in `message_history.rs` and is used by the capacity cap logic (which remains in `GameState`).
- [ ] `Serialize` / `Deserialize` implemented transparently (delegates to `Vec<Message>`).
- [ ] `Default` yields empty history.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** None
**Files touched:** `src/model/message_history.rs`
**Estimated scope:** Small (1 file)

---

#### Task 2: Update `src/model/mod.rs`

**Description:** Add `pub mod message_history;` to the module declarations.

**Acceptance criteria:**
- [ ] `message_history` module is declared.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 1
**Files touched:** `src/model/mod.rs`
**Estimated scope:** XS

---

#### Task 3: Update `NarrativeState` and `GameState`

**Description:** Replace `messages: Vec<Message>` with `history: MessageHistory` in `NarrativeState`. Update `NarrativeState::history()` to delegate. Update `NarrativeState::from_snapshot` to initialize `history` as empty. Update `GameState` methods to delegate to `self.narrative.history`.

**Acceptance criteria:**
- [ ] `NarrativeState.messages` field removed; `history: MessageHistory` added.
- [ ] `GameState.push_message`, `add_log`, `add_input`, `edit_log`, `delete_last_log`, `get_log`, `get_last_ai_response_index`, `get_last_input_index`, `get_last_input_text`, `is_last_ai_response_event_continuation` delegate to `self.narrative.history`.
- [ ] `push_message` still absorbs `pending_location`/`pending_event` before calling `history.append(message)`.

**Verification:**
- [ ] `cargo check` passes.
- [ ] Existing tests in `model/state_tests.rs` still compile.

**Dependencies:** Task 2
**Files touched:** `src/model/state.rs`
**Estimated scope:** Small (1 file)

---

### Checkpoint: After Tasks 1-3

- [ ] `cargo check` clean.
- [ ] `cargo test` in `model/` passes.

---

### Phase 2: Migrate Call Sites

#### Task 4: Migrate `application/game_service/` call sites

**Description:** Update `retry.rs`, `helpers.rs`, and their tests to use `state.narrative.history` methods instead of direct `messages` access.

Specific changes:
- `retry.rs:84` — `state.narrative.messages = messages` → `state.narrative.history.replace(messages)`
- `helpers.rs:35` — `state.narrative.messages = msgs` → `state.narrative.history.replace(msgs)`
- `helpers.rs:73` — `state.narrative.messages.iter_mut()` → `state.narrative.history.iter_mut()`
- `helpers.rs:89` — `state.narrative.messages.retain(...)` → `state.narrative.history.retain(...)`
- `helpers_tests.rs:137-138` — `state.narrative.messages[...]` → `state.narrative.history.as_slice()[...]`
- `retry_tests.rs` — all `state.narrative.messages.last_mut()` → `state.narrative.history.last_mut()`

**Acceptance criteria:**
- [ ] No direct `narrative.messages` references remain in `application/game_service/`.
- [ ] `cargo test` passes for `application/game_service/` tests.

**Verification:**
- [ ] `cargo test application::game_service`

**Dependencies:** Task 3
**Files touched:**
- `src/application/game_service/retry.rs`
- `src/application/game_service/helpers.rs`
- `src/application/game_service/helpers_tests.rs`
- `src/application/game_service/retry_tests.rs`
**Estimated scope:** Medium (4 files)

---

#### Task 5: Migrate `server/` and `bootstrap/` call sites

**Description:** Update `server/mod.rs`, `server/fragments/checkpoint.rs`, `bootstrap/run.rs`, and `bootstrap/scenario.rs`.

Specific changes:
- `server/mod.rs:132` — `state.narrative.messages.clone()` → `state.narrative.history.iter().cloned().collect()`
- `server/mod.rs:255` — `game_state.narrative.messages = messages` → `game_state.narrative.history.replace(messages)`
- `checkpoint.rs:73` — same as above
- `bootstrap/run.rs:71` — `state.narrative.messages.iter_mut()` → `state.narrative.history.iter_mut()`
- `bootstrap/run.rs:125` — `state.narrative.messages = msgs` → `state.narrative.history.replace(msgs)`
- `bootstrap/scenario.rs:20` — `state.add_log(...)` unchanged (delegator still exists)
- `bootstrap/run.rs:158` — `state.add_log(...)` unchanged

**Acceptance criteria:**
- [ ] No direct `narrative.messages` references remain in `server/` or `bootstrap/`.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 3
**Files touched:**
- `src/server/mod.rs`
- `src/server/fragments/checkpoint.rs`
- `src/bootstrap/run.rs`
**Estimated scope:** Small (3 files)

---

#### Task 6: Migrate `engine/` and `test_support/` call sites

**Description:** Update `engine/state_diagnostics.rs` and `test_support/context.rs`.

Specific changes:
- `engine/state_diagnostics.rs:71-72` — delegator methods unchanged (`state.get_last_ai_response_index()` etc.)
- `test_support/context.rs:17` and `:53` — `state.narrative.messages.clone()` → `state.narrative.history.iter().cloned().collect()`

**Acceptance criteria:**
- [ ] No direct `narrative.messages` references remain in `engine/` or `test_support/`.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 3
**Files touched:**
- `src/test_support/context.rs`
**Estimated scope:** XS (1 file)

---

#### Task 7: Migrate `model/state_tests.rs`

**Description:** Update all test code that touches `state.narrative.messages` directly.

Specific changes:
- `state.add_log(...)` calls — unchanged (delegators still exist).
- `state.narrative.messages.len()` → `state.narrative.history.len()`
- `state.narrative.messages.last()` → `state.narrative.history.last()`
- `state.narrative.messages.is_empty()` → `state.narrative.history.is_empty()`
- `state.narrative.messages[0]` → `state.narrative.history.as_slice()[0]`

**Acceptance criteria:**
- [ ] All `model/state_tests.rs` tests compile and pass.

**Verification:**
- [ ] `cargo test model::state_tests`

**Dependencies:** Task 3
**Files touched:** `src/model/state_tests.rs`
**Estimated scope:** Small (1 file)

---

### Checkpoint: After Tasks 4-7

- [ ] `cargo test` passes across the entire crate.
- [ ] `cargo clippy` passes.
- [ ] No `narrative.messages` string remains in the codebase (grep to confirm).

---

### Phase 3: Add Unit Tests for MessageHistory

#### Task 8: Create `src/model/message_history_tests.rs`

**Description:** Move message-history-specific test logic from `state_tests.rs` into dedicated `MessageHistory` unit tests. Add tests for new methods (`replace`, `retain`, `from_messages`, etc.).

Tests to port/adapt:
- Capacity cap (still exercised via `GameState::add_log`, but `MessageHistory::len()` boundary)
- `edit` success and failure
- `delete_last` success and failure
- `last_ai_response_index` / `last_input_index`
- `is_last_ai_response_event_continuation`
- `replace` / `retain` / `clear`

**Acceptance criteria:**
- [ ] `message_history_tests.rs` covers all public `MessageHistory` methods.
- [ ] Tests pass.

**Verification:**
- [ ] `cargo test model::message_history_tests`

**Dependencies:** Tasks 1-7
**Files touched:** `src/model/message_history_tests.rs`, `src/model/mod.rs`
**Estimated scope:** Small (2 files)

---

### Checkpoint: Complete

- [ ] All acceptance criteria met.
- [ ] `cargo test`, `cargo clippy`, `python build.py` all pass.
- [ ] No `narrative.messages` references remain anywhere in `src/`.

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Snapshot serialization breakage | Low | `MessageHistory` serializes as `Vec<Message>` transparently via custom `Serialize`/`Deserialize`. `NarrativeSnapshot` already excludes messages, so snapshot format is unchanged. |
| Test compilation churn | Medium | Systematic grep-and-replace per task above; verify with `cargo test` after each phase. |
| Scope creep | Medium | Strictly exclude non-message fields (`input_buffer`, `last_trigger`, `pending_*`). `push_message` stays in `GameState` because it reads `pending_location`/`pending_event`. |
| Encapsulation leak via `messages_mut` | Low | **Deliberately omitted** `messages_mut()` from the API. Use `replace()`, `iter_mut()`, `retain()`, `last_mut()` instead. |

---

## Open Questions

- Should we eventually remove the `GameState` delegator methods (`add_log`, `edit_log`, etc.) and migrate all ~30 call sites to `state.narrative.history.append(...)`? **Recommendation:** Keep delegators for this plan to minimize churn; remove them in a follow-up cleanup if desired.
- Should `MessageHistory` own the `pending_location`/`pending_event` absorption logic? **Recommendation:** No — those fields live in `NarrativeState`, and moving them would require `MessageHistory` to know about headers, which is a larger refactor.
