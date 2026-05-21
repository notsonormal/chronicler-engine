# Implementation Plan: Multi-Game Support Review Fixes

## Overview

Address the two correctness issues identified in the multi-game support code review, plus three small code-health improvements. All changes are scoped to `chronicler_engine/`.

## Architecture Decisions

- **Latest-game heuristic:** Use the most recent message timestamp per game as the primary sort key for "latest game", with `games.updated_at` as a fallback. This accurately reflects which game was actually played most recently, rather than which was last switched to.
- **Transaction boundary:** `delete_game` must be atomic. A rusqlite `Transaction` wraps the three cascading `DELETE`s so partial failure cannot leave orphaned snapshots or messages.

## Task List

### Task 1: Fix `find_latest_game_for_world` ordering

**Description:**
Replace the `ORDER BY updated_at DESC` query in `bootstrap/run.rs` with one that orders by the most recent message timestamp for each game. If a game has no messages, fall back to `updated_at`.

**Acceptance criteria:**
- [ ] `find_latest_game_for_world` returns the game whose most recent message is newest.
- [ ] Games with no messages fall back to `updated_at`.
- [ ] The query still filters by `world_name`.

**Verification:**
- [ ] New unit test in `bootstrap/run.rs` (or a new `bootstrap/run_tests.rs`) creates two games, saves messages to only one, and asserts the messaged game is returned as latest.
- [ ] Existing tests still pass.

**Files likely touched:**
- `chronicler_engine/src/bootstrap/run.rs`
- `chronicler_engine/src/bootstrap/run_tests.rs` (new, or inline `#[cfg(test)]` module)

**Estimated scope:** Small

---

### Task 2: Wrap `delete_game` in a SQLite transaction

**Description:**
Convert the three sequential `DELETE` statements in `SqliteGameStorage::delete_game` into an atomic transaction. If any step fails, roll back and return the error.

**Acceptance criteria:**
- [ ] `delete_game` begins a transaction, runs the three deletes, then commits.
- [ ] Partial failures are rolled back and return an error.

**Verification:**
- [ ] `test_delete_game_cascades` in `tests/snapshot_storage_tests.rs` still passes.
- [ ] Full test suite passes.

**Files likely touched:**
- `chronicler_engine/src/storage/snapshot_storage.rs`

**Estimated scope:** XS

---

### Checkpoint: After Tasks 1–2
- [ ] All tests pass
- [ ] `cargo clippy` is clean
- [ ] The two correctness issues from review are resolved

---

### Task 3: Simplify `generate_game_name` to max+1

**Description:**
Replace the gap-filling loop in `model/game.rs` with a simpler algorithm: parse the trailing `_N` segment of every existing name that matches today's `{WorldName}_{Date}_` prefix, find the maximum `N`, and return `{WorldName}_{Date}_{N+1}`. If no matching names exist, return `_1`.

**Acceptance criteria:**
- [ ] `generate_game_name("Redmist", &["Redmist_2026-05-21_1", "Redmist_2026-05-21_3"])` returns `Redmist_2026-05-21_4`.
- [ ] `generate_game_name("Redmist", &[])` returns `Redmist_2026-05-21_1`.
- [ ] Names from other worlds or dates are ignored.

**Verification:**
- [ ] Update `game_tests.rs` to match the new max+1 semantics.
- [ ] All game name tests pass.

**Files likely touched:**
- `chronicler_engine/src/model/game.rs`
- `chronicler_engine/src/model/game_tests.rs`

**Estimated scope:** XS

---

### Task 4: Extract `game_id()` helper

**Description:**
Add a private `fn game_id(&self) -> u64` helper to `SqliteGameStorage` (and reuse `do_current_game_id` in `InMemoryGameStorage`) to replace the ~20 inline `self.game_id.load(Ordering::SeqCst)` calls.

**Acceptance criteria:**
- [ ] No inline `AtomicU64::load` calls remain in storage method bodies.
- [ ] Behavior is unchanged.

**Verification:**
- [ ] `cargo clippy` clean.
- [ ] All storage tests pass.

**Files likely touched:**
- `chronicler_engine/src/storage/snapshot_storage.rs`
- `chronicler_engine/src/test_support/in_memory_storage.rs`

**Estimated scope:** Small

---

### Task 5: Add error-path test for `list_games_fragment`

**Description:**
Add a test that verifies `GET /fragment/games` returns `500 INTERNAL_SERVER_ERROR` (or the appropriate error response) when `SnapshotStorage::list_games` fails.

**Acceptance criteria:**
- [ ] A minimal mock storage that fails `list_games` is created inline in the test file.
- [ ] The test asserts the correct status code and error body.

**Verification:**
- [ ] New test passes.
- [ ] Existing fragment tests pass.

**Files likely touched:**
- `chronicler_engine/tests/components/fragment.rs`

**Estimated scope:** Small

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| SQL subquery for message timestamp is slow on large DBs | Low | Local SQLite with handfuls of games; add index on `messages(game_id, timestamp)` if profiling shows it matters. |
| Transaction rollback edge cases | Low | Rusqlite `Transaction` drops and rolls back automatically on error; explicit `?` propagation ensures this. |
| `game_id()` refactor touches many lines | Low | Pure refactor with no behavior change; comprehensive test coverage already exists. |

## Open Questions

- None. The user has already specified the heuristic for Task 1 (most recent message).
