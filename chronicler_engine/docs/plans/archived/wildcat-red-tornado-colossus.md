# Plan: MessageStorage Mutates Message Directly (Swipe Blast Radius)

## Overview

`SqliteGameStorage` implements both `SnapshotStorage` and `MessageStorage` in a single struct. `MessageStorage::insert_message` takes `&mut Message` and mutates `msg.id` directly. This means snapshot changes force message recompilation, and the hidden side-effect is inconsistent with `SnapshotStorage::save(&GameStateSnapshot) -> Result<u64, _>`.

We will split the sqlite and in-memory storage into separate repository structs, change `insert_message` to return the generated ID, and update all callers.

## Architecture Decisions

- **No `NewMessageDto`**: Use `&Message -> Result<u64, EngineError>` instead. This matches the existing `SnapshotStorage::save` pattern, avoids introducing a redundant DTO type, and achieves the same decoupling with less code.
- **Remove `reset`**: It exists only on concrete types and is only used in tests. Tests create fresh in-memory DBs per test anyway. `delete_game` on `SnapshotStorage` already covers cross-table cleanup.
- **Keep `delete_game` on `SnapshotStorage`**: It already deletes both snapshots and messages in a transaction. Moving it would require a third "game repository" type — over-engineering for this change.

---

## Phase 1: Foundation — Trait Change and Sqlite Repo Split

### Task 1: Change `MessageStorage` trait signature

**Description:** Narrow `insert_message` to accept `&Message` and return `Result<u64, EngineError>`.

**Files touched:**
- `src/storage/message_storage.rs`

**Acceptance criteria:**
- [ ] `insert_message` signature is `fn insert_message(&self, msg: &Message) -> Result<u64, EngineError>;`
- [ ] All other trait methods unchanged

**Verification:**
- [ ] `cargo check --lib` shows only expected errors (impls not yet updated)

**Scope:** XS

---

### Task 2: Create `SqliteMessageRepository`

**Description:** Extract all `MessageStorage` methods from `SqliteGameStorage` into a new `SqliteMessageRepository` struct. Update `insert_message` to return the generated ID instead of mutating.

**Files touched:**
- `src/storage/message_storage.rs`

**Acceptance criteria:**
- [ ] `SqliteMessageRepository { pool: DbPool, game_id: AtomicU64 }` exists
- [ ] `impl MessageStorage for SqliteMessageRepository` with all methods
- [ ] `insert_message` returns `conn.last_insert_rowid() as u64`
- [ ] `set_game_id` / `current_game_id` work correctly

**Verification:**
- [ ] `cargo check --lib` for `chronicler_engine::storage::message_storage` module passes

**Dependencies:** Task 1
**Scope:** M

---

### Task 3: Rename `SqliteGameStorage` → `SqliteSnapshotRepository`

**Description:** Remove message methods and `reset` from the old struct. Rename it to `SqliteSnapshotRepository`.

**Files touched:**
- `src/storage/snapshot_storage.rs`

**Acceptance criteria:**
- [ ] `SqliteGameStorage` no longer exists
- [ ] `SqliteSnapshotRepository` implements `SnapshotStorage` with all original methods
- [ ] `reset()` method removed
- [ ] `delete_game` still performs cross-table deletion in a transaction

**Verification:**
- [ ] `cargo check --lib` for `chronicler_engine::storage::snapshot_storage` module passes

**Dependencies:** Task 2
**Scope:** M

---

## Checkpoint: After Tasks 1-3
- [ ] `cargo check --lib` passes for core storage modules
- [ ] No references to `SqliteGameStorage` remain in `src/storage/`

---

## Phase 2: Test Support — Split In-Memory Storage

### Task 4: Split `InMemoryGameStorage`

**Description:** Replace `InMemoryGameStorage` with `InMemorySnapshotRepository` + `InMemoryMessageRepository`. Update `insert_message` to return ID.

**Files touched:**
- `src/test_support/in_memory_storage.rs`

**Acceptance criteria:**
- [ ] `InMemoryGameStorage` no longer exists
- [ ] `InMemorySnapshotRepository` implements `SnapshotStorage`
- [ ] `InMemoryMessageRepository` implements `MessageStorage`
- [ ] `insert_message` returns `*next_id` without mutating input
- [ ] `reset()` removed from both

**Verification:**
- [ ] `cargo check --lib` for `chronicler_engine::test_support` passes

**Dependencies:** Task 1
**Scope:** M

---

### Task 5: Update `test_support/context.rs`

**Description:** Update all test context builders to create both repos separately.

**Files touched:**
- `src/test_support/context.rs`

**Acceptance criteria:**
- [ ] `make_test_context` creates `InMemorySnapshotRepository` + `InMemoryMessageRepository`
- [ ] `make_test_context_without_snapshot` same
- [ ] `make_test_context_with_sqlite` creates `SqliteSnapshotRepository` + `SqliteMessageRepository`
- [ ] `GameServiceContext` fields populated correctly

**Verification:**
- [ ] `cargo check --lib` passes

**Dependencies:** Task 4
**Scope:** S

---

## Checkpoint: After Tasks 4-5
- [ ] `cargo check --lib` passes for all library code
- [ ] No references to `InMemoryGameStorage` remain

---

## Phase 3: Production Callers

### Task 6: Update core application code

**Description:** Update all production `insert_message` call sites to capture the returned ID and assign it explicitly. Use two-statement pattern to avoid borrow checker issues.

**Files touched:**
- `src/bootstrap/run.rs`
- `src/application/context.rs`
- `src/application/application_service.rs`

**Acceptance criteria:**
- [ ] `run.rs` creates `SqliteSnapshotRepository` + `SqliteMessageRepository` separately from `db_pool`
- [ ] All `insert_message(msg)?` calls changed to `let id = ...insert_message(&*msg)?; msg.id = id;`
- [ ] No `&mut Message` passed to `insert_message`

**Verification:**
- [ ] `cargo check --lib` passes

**Dependencies:** Task 3
**Scope:** S

---

### Task 7: Update server test setup

**Description:** Update `create_app_for_testing_with_settings` to use split in-memory repos.

**Files touched:**
- `src/server/mod.rs`

**Acceptance criteria:**
- [ ] Creates `InMemorySnapshotRepository` + `InMemoryMessageRepository` separately
- [ ] `insert_message` calls updated (cloned messages, no mutation needed)

**Verification:**
- [ ] `cargo check --lib` passes

**Dependencies:** Task 4
**Scope:** XS

---

## Checkpoint: After Tasks 6-7
- [ ] `cargo check --lib` passes for entire library
- [ ] No `&mut` passed to `insert_message` in production code

---

## Phase 4: Tests and Mocks

### Task 8: Update mock `MessageStorage` implementations

**Description:** Update mock structs that implement `MessageStorage` to match the new signature.

**Files touched:**
- `src/application/action_pipeline/retry_tests.rs`
- `tests/components/fragment.rs`

**Acceptance criteria:**
- [ ] All mock `insert_message` methods have new signature
- [ ] Delegate calls updated (`&mut` removed where delegating)

**Verification:**
- [ ] `cargo check --tests` shows no errors in these files

**Dependencies:** Task 1
**Scope:** S

---

### Task 9: Update unit tests

**Description:** Update tests in `src/` that use `insert_message` or `reset`.

**Files touched:**
- `src/storage/snapshot_storage_tests.rs`
- `src/test_support/in_memory_storage_tests.rs`
- `src/application/context_tests.rs`

**Acceptance criteria:**
- [ ] `snapshot_storage_tests.rs`: references `SqliteSnapshotRepository`, `reset` tests removed/rewritten
- [ ] `in_memory_storage_tests.rs`: references split repos, `test_reset` removed
- [ ] `context_tests.rs`: `insert_message` call uses `&msg` (cloned message)

**Verification:**
- [ ] `cargo test --lib` for affected modules passes

**Dependencies:** Tasks 3, 4, 5
**Scope:** M

---

### Task 10: Update integration tests

**Description:** Update all integration test files that create storage or call `insert_message`.

**Files touched:**
- `tests/snapshot_storage_tests.rs`
- `tests/flow_mock/retry_main.rs`
- `tests/helpers/pipeline_helpers.rs`
- `tests/components/misc.rs`
- `tests/action_pipeline/retry.rs`

**Acceptance criteria:**
- [ ] `snapshot_storage_tests.rs`: uses `SqliteSnapshotRepository`, `reset` tests removed
- [ ] `retry_main.rs`: creates split sqlite repos, `storage.reset()` replaced
- [ ] `pipeline_helpers.rs`: `save_state` uses `&msg` (cloned messages)
- [ ] `misc.rs`: creates split repos, `insert_message` calls updated
- [ ] `retry.rs`: `insert_message` calls updated

**Verification:**
- [ ] `cargo test --test snapshot_storage_tests` passes
- [ ] `cargo test --test retry` passes
- [ ] `cargo test --test misc` passes
- [ ] `cargo test --test fragment` passes
- [ ] `cargo test --test retry_main` passes

**Dependencies:** Tasks 3, 4, 5, 6, 7, 8
**Scope:** L (but mechanical; many small edits)

---

## Checkpoint: Complete
- [ ] `cd chronicler_engine && python build.py` passes (fmt + clippy + tests + coverage)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Borrow-checker at `&*msg` call sites | Med | Use two-statement pattern: `let id = repo.insert_message(&*msg)?; msg.id = id;` |
| Tests that relied on `reset` | Low | `reset` was test-only; most tests use fresh DBs per test |
| Large mechanical diff | Med | Tasks are vertically sliced; each checkpoint validates a working subset |
| Missing a call site | Med | `cargo check --tests` will catch any remaining `&mut` borrow |
