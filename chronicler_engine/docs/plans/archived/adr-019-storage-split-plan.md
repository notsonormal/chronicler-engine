# Implementation Plan: One Table Per Storage Module

## Overview

Split chronicler_engine's multi-table storage modules so each physical SQLite table is owned by exactly one Rust storage trait and repository. This prevents domain-logic shortcuts (like creating new messages just to get new swipes) by making cross-table operations explicit at the caller level.

## Architecture Decisions

- **One table → one `xxx_storage.rs` module** containing a trait + SQLite repository + in-memory test impl.
- **No storage method may touch more than one table.** Cross-table operations live in `GameServiceContext` convenience methods or explicit caller coordination.
- **No caller-level SQLite transactions** (DbPool is a single `Mutex<Connection>`; recursive locking deadlocks). Each storage call is individually atomic.
- **`ON DELETE CASCADE` FKs** in schema for `games` → `game_state_snapshots` and `games` → `messages`, so `delete_game` becomes a single-row delete.
- **Helpers on `GameServiceContext`** (`load_messages`, `update_message_text`, `migrate_swipes`) minimize caller churn during the transition.

## Dependency Graph

```
ADR-019
    │
    ├── Schema migration (CASCADE DELETE FKs)
    │       │
    │       └── Simplifies GameStorage::delete_game
    │
    ├── MessageSwipeStorage trait + impls
    │       │
    │       ├── GameServiceContext helpers (load_messages, migrate_swipes)
    │       │       │
    │       │       └── Refactor MessageStorage trait + callers
    │       │
    │       └── Tests
    │
    ├── GameStorage trait + impls
    │       │
    │       ├── GameServiceContext field + wiring
    │       │       │
    │       │       └── Refactor SnapshotStorage trait + callers
    │       │
    │       └── Tests
    │
    └── Full validation
```

## Task List

### Phase 1: Documentation

#### Task 1: Write ADR-019
**Description:** Document the one-table-per-storage-module rule, the trade-offs discussed (no caller-level transactions, GameServiceContext helpers), and the specific splits for messages/swipes and games/snapshots.

**Acceptance criteria:**
- [ ] ADR follows existing `adr-NNN-title.md` format in `chronicler_engine/docs/adr/`
- [ ] References ADR-017 (Message Swipes) and ADR-008 (SQLite Snapshot Persistence)
- [ ] Lists all trait API changes (`insert_message`, `load_messages`, `update_message`, `migrate_swipes`, `delete_game`)

**Verification:**
- [ ] File renders correctly in markdown preview

**Files likely touched:**
- `chronicler_engine/docs/adr/adr-019-one-table-per-storage-module.md`

**Estimated scope:** Small

---

### Phase 2: Schema Foundation

#### Task 2: Add CASCADE DELETE Foreign Keys
**Description:** Add `ON DELETE CASCADE` FK constraints to `messages.game_id` → `games(id)` and `game_state_snapshots.game_id` → `games(id)` in a new migration (v9). This lets `GameStorage::delete_game` delete only the `games` row.

**Acceptance criteria:**
- [ ] New migration v9 runs successfully on existing databases
- [ ] `game_state_snapshots` rows are auto-deleted when parent game is deleted
- [ ] `messages` rows are auto-deleted when parent game is deleted
- [ ] `message_swipes` already has CASCADE; verify chain works end-to-end

**Verification:**
- [ ] `storage/db_tests.rs` or new test verifies CASCADE behavior

**Files likely touched:**
- `chronicler_engine/src/storage/db.rs`
- `chronicler_engine/src/storage/db_tests.rs`

**Estimated scope:** Small

---

### Phase 3: Message / Swipe Split

#### Task 3: Create `MessageSwipeStorage` Trait and Implementations
**Description:** Create `src/storage/message_swipe_storage.rs` with `MessageSwipeStorage` trait (`insert_swipe`, `shift_swipe_indices`, `load_swipes_for_message`, `load_swipes_for_messages`). Implement `SqliteMessageSwipeRepository` and `InMemoryMessageSwipeStorage`. Add module declaration to `storage/mod.rs`.

**Acceptance criteria:**
- [ ] Trait compiles with all methods
- [ ] SQLite implementation passes new unit tests
- [ ] In-memory implementation passes new unit tests
- [ ] No existing code broken (new trait, not yet wired)

**Verification:**
- [ ] `cargo test -p chronicler_engine message_swipe` passes
- [ ] `cargo check` clean

**Files likely touched:**
- `chronicler_engine/src/storage/message_swipe_storage.rs` (new)
- `chronicler_engine/src/storage/message_swipe_storage_tests.rs` (new)
- `chronicler_engine/src/storage/mod.rs`
- `chronicler_engine/src/test_support/in_memory_storage.rs` (add `InMemoryMessageSwipeStorage`)

**Estimated scope:** Medium

---

#### Task 4: Add Swipe Storage to `GameServiceContext` and Add Helpers
**Description:** Add `message_swipe_storage: Arc<dyn MessageSwipeStorage>` to `GameServiceContext`, `AppState`, `ServerResources`, and bootstrap. Add convenience methods to `GameServiceContext`: `load_messages()`, `update_message_text(id, text)`, `migrate_swipes(...)`. Initial implementations delegate to existing `MessageStorage` methods so callers can migrate incrementally.

**Acceptance criteria:**
- [ ] `GameServiceContext` has new field
- [ ] Bootstrap and server wiring compile
- [ ] `load_messages()` helper returns same result as old `message_storage.load_messages()`
- [ ] `update_message_text()` delegates to existing `message_storage.update_message()`
- [ ] `migrate_swipes()` delegates to existing `message_storage.migrate_swipes()`

**Verification:**
- [ ] `cargo check` clean
- [ ] Existing tests still pass

**Files likely touched:**
- `chronicler_engine/src/application/context.rs`
- `chronicler_engine/src/server/mod.rs`
- `chronicler_engine/src/bootstrap/run.rs`
- `chronicler_engine/src/test_support/context.rs`

**Estimated scope:** Medium

---

#### Task 5: Migrate Callers to `GameServiceContext` Helpers
**Description:** Update all production and test callers of `load_messages()`, `update_message()`, and `migrate_swipes()` to use the new `GameServiceContext` helpers. This decouples callers from the trait methods that will disappear.

**Acceptance criteria:**
- [ ] No production code calls `ctx.message_storage.load_messages()`
- [ ] No production code calls `ctx.message_storage.update_message(...)`
- [ ] No production code calls `ctx.message_storage.migrate_swipes(...)`
- [ ] All call sites use `ctx.load_messages()`, `ctx.update_message_text(...)`, `ctx.migrate_swipes(...)`

**Verification:**
- [ ] `cargo check` clean
- [ ] Existing tests pass

**Files likely touched:**
- `chronicler_engine/src/application/application_service.rs`
- `chronicler_engine/src/application/action_pipeline/retry.rs`
- `chronicler_engine/src/application/context.rs` (if any direct calls)
- `chronicler_engine/src/bootstrap/run.rs`
- `chronicler_engine/src/application/action_pipeline/retry_tests.rs`
- `chronicler_engine/src/application/context_tests.rs`

**Estimated scope:** Medium

---

#### Task 6: Refactor `MessageStorage` Trait and Implementations
**Description:** Remove swipe-related methods from `MessageStorage`. Change `insert_message` to accept message metadata (no swipes). Remove `update_message`, `load_messages`, `migrate_swipes` from trait. Update `SqliteMessageRepository` and `InMemoryMessageRepository`. Update `GameServiceContext` helpers to use the new split (e.g., `load_messages` now calls both storages and assembles; `insert_message` callers must separately call `insert_swipe`).

**Acceptance criteria:**
- [ ] `MessageStorage` trait only touches `messages` table
- [ ] `insert_message` no longer accepts `Message` with swipes
- [ ] `SqliteMessageRepository` updated
- [ ] `InMemoryMessageRepository` updated
- [ ] `GameServiceContext::load_messages()` assembles from both storages
- [ ] `GameServiceContext::update_message_text()` reads active index + updates swipe
- [ ] `GameServiceContext::migrate_swipes()` coordinates both storages
- [ ] All `insert_message` callers also call `insert_swipe` for the first swipe

**Verification:**
- [ ] `cargo test -p chronicler_engine` passes
- [ ] `cargo check` clean

**Files likely touched:**
- `chronicler_engine/src/storage/message_storage.rs`
- `chronicler_engine/src/storage/message_storage_tests.rs`
- `chronicler_engine/src/test_support/in_memory_storage.rs`
- `chronicler_engine/src/application/context.rs`
- `chronicler_engine/src/application/application_service.rs`
- `chronicler_engine/src/bootstrap/run.rs`
- `chronicler_engine/src/application/action_pipeline/retry_tests.rs`
- `chronicler_engine/src/application/action_pipeline/pipeline_tests.rs`

**Estimated scope:** Large (but localized to message domain)

---

### Checkpoint: After Tasks 1-6
- [ ] All message/swipe tests pass
- [ ] `cargo check` clean
- [ ] Retry logic still functions (manual verification or existing retry tests)

---

### Phase 4: Game / Snapshot Split

#### Task 7: Create `GameStorage` Trait and Implementations
**Description:** Create `src/storage/game_storage.rs` with `GameStorage` trait (`list_games`, `create_game`, `delete_game`, `get_game`, `set_game_id`, `current_game_id`). Implement `SqliteGameRepository` and `InMemoryGameRepository`. Use CASCADE DELETE from Task 2 so `delete_game` is a single-row delete. Add module declaration to `storage/mod.rs`.

**Acceptance criteria:**
- [ ] Trait compiles with all methods
- [ ] SQLite implementation passes new unit tests
- [ ] In-memory implementation passes new unit tests
- [ ] `delete_game` uses single `DELETE FROM games WHERE id = ?` (CASCADE handles children)
- [ ] No existing code broken (new trait, not yet wired)

**Verification:**
- [ ] `cargo test -p chronicler_engine game_storage` passes
- [ ] `cargo check` clean

**Files likely touched:**
- `chronicler_engine/src/storage/game_storage.rs` (new)
- `chronicler_engine/src/storage/game_storage_tests.rs` (new)
- `chronicler_engine/src/storage/mod.rs`
- `chronicler_engine/src/test_support/in_memory_storage.rs` (add `InMemoryGameRepository`)

**Estimated scope:** Medium

---

#### Task 8: Add Game Storage to `GameServiceContext` and Wire Bootstrap/Server
**Description:** Add `game_storage: Arc<dyn GameStorage>` to `GameServiceContext`, `AppState`, `ServerResources`, and bootstrap. Update `as_game_service_context()` and test context builders.

**Acceptance criteria:**
- [ ] `GameServiceContext` has new field
- [ ] Bootstrap and server wiring compile
- [ ] Existing tests still pass (trait not yet used by callers)

**Verification:**
- [ ] `cargo check` clean
- [ ] Existing tests pass

**Files likely touched:**
- `chronicler_engine/src/application/context.rs`
- `chronicler_engine/src/server/mod.rs`
- `chronicler_engine/src/bootstrap/run.rs`
- `chronicler_engine/src/test_support/context.rs`

**Estimated scope:** Small

---

#### Task 9: Refactor `SnapshotStorage` Trait and Migrate Callers
**Description:** Remove game-related methods from `SnapshotStorage` trait. Update `SqliteSnapshotRepository` and `InMemorySnapshotRepository`. Migrate all callers of `list_games`, `create_game`, `delete_game`, `get_game`, `set_game_id` on `SnapshotStorage` to use `game_storage` instead. Update `GameServiceContext::set_game_id` helper if needed.

**Acceptance criteria:**
- [ ] `SnapshotStorage` trait only touches `game_state_snapshots` table
- [ ] `SqliteSnapshotRepository` updated
- [ ] `InMemorySnapshotRepository` updated
- [ ] No production code calls game methods on `snapshot_storage`
- [ ] All game callers use `ctx.game_storage`

**Verification:**
- [ ] `cargo test -p chronicler_engine` passes
- [ ] `cargo check` clean

**Files likely touched:**
- `chronicler_engine/src/storage/snapshot_storage.rs`
- `chronicler_engine/src/storage/snapshot_storage_tests.rs`
- `chronicler_engine/src/test_support/in_memory_storage.rs`
- `chronicler_engine/src/application/application_service.rs`
- `chronicler_engine/src/application/context.rs`
- `chronicler_engine/src/bootstrap/run.rs`
- `chronicler_engine/src/server/fragments/games.rs` (if any direct calls)

**Estimated scope:** Medium

---

### Checkpoint: After Tasks 7-9
- [ ] All storage tests pass
- [ ] `cargo check` clean
- [ ] Game CRUD still functions (manual verification or existing tests)

---

### Phase 5: Cleanup and Validation

#### Task 10: Remove Deprecated Methods and Clean Up
**Description:** Remove any remaining dead code: old cross-table methods that are no longer on traits but may still exist as private helpers. Update `storage/mod.rs` exports if needed. Ensure no `TODO` or `FIXME` left from the transition.

**Acceptance criteria:**
- [ ] No dead code in storage modules
- [ ] `cargo clippy` clean (no unused code warnings)

**Verification:**
- [ ] `cargo clippy` passes
- [ ] `cargo check` clean

**Files likely touched:**
- `chronicler_engine/src/storage/message_storage.rs`
- `chronicler_engine/src/storage/snapshot_storage.rs`
- `chronicler_engine/src/test_support/in_memory_storage.rs`

**Estimated scope:** Small

---

#### Task 11: Full Validation
**Description:** Run the project's full validation suite: `python build.py` (fmt + clippy + tests + coverage).

**Acceptance criteria:**
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean
- [ ] All tests pass
- [ ] Coverage acceptable (no major drops)

**Verification:**
- [ ] `cd chronicler_engine && python build.py` succeeds

**Files likely touched:** None (validation only)

**Estimated scope:** Small

---

### Phase 6: Pipeline Retry Refactor (Post-Storage Split)

#### Task 12: Refactor Retry to Avoid `migrate_swipes`
**Description:** Rewrite `retry.rs` so it no longer creates a new message and migrates swipes. Instead: keep the existing message, call `insert_swipe` for the new generation, and `update_active_swipe` to activate it. Remove the `migrate_swipes` helper from `GameServiceContext`.

**Acceptance criteria:**
- [ ] Retry logic no longer calls `migrate_swipes`
- [ ] Retry logic uses `insert_swipe` + `update_active_swipe`
- [ ] `migrate_swipes` method removed from `GameServiceContext`
- [ ] All retry tests pass

**Verification:**
- [ ] `cargo test -p chronicler_engine retry` passes
- [ ] Manual or integration test: retry a narration, verify old swipes preserved + new swipe added

**Files likely touched:**
- `chronicler_engine/src/application/action_pipeline/retry.rs`
- `chronicler_engine/src/application/action_pipeline/retry_tests.rs`
- `chronicler_engine/src/application/context.rs`

**Estimated scope:** Medium

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `cargo check` fails for extended period due to trait changes | High | Use helper methods on `GameServiceContext` as abstraction layer; migrate callers incrementally before breaking traits |
| Test flakiness from non-atomic cross-storage operations | Medium | Each storage call is a single SQLite statement (atomic). Gap between calls is tiny. Retry logic refactor (Task 12) reduces multi-step operations |
| Missing a caller during migration | Medium | Use `grep` to find all `message_storage.` and `snapshot_storage.` call sites; verify none reference removed methods |
| Bootstrap or server wiring mismatch | Medium | `bootstrap/run.rs` and `server/mod.rs` are small, explicit files; compile errors surface wiring issues immediately |

## Open Questions

- None remaining from grilling session. All dependencies resolved.
