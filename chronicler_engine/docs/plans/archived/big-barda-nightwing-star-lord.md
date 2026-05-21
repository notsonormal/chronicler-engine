# Plan: Multi-Game Support for Chronicler Engine

## Overview
Implement **multi-game support** using the existing `games` table. A player can have multiple parallel games (one per world or multiple per world), each with its own snapshots and messages. Games are named automatically on creation (`{WorldName}_{Date}_N`). Remove checkpoint functionality entirely — games replace checkpoints as the primary save-state mechanism.

## Architecture Decisions
- **Use existing `games` table**: Add a `name` column for display. No new table needed.
- **Mutable `game_id` on storage traits**: Add `set_game_id` to `SnapshotStorage` and `MessageStorage` so game switching doesn't require recreating `AppState`.
- **Checkpoints removed entirely**: The user confirmed checkpoints feel non-functional. Multiple games with per-turn snapshots provide equivalent functionality.
- **Auto-naming on first launch**: When no games exist for the selected world, startup auto-creates one. Users can also create games manually via the UI.

## Dependency Graph

```
Migration v5 (schema: games.name, drop checkpoints)
    │
    ├── Domain model (Game)
    │       │
    │       ├── Storage traits (+set_game_id, +game CRUD, -checkpoints)
    │       │       │
    │       │       ├── InMemoryGameStorage (test parity)
    │       │       │
    │       │       └── SqliteGameStorage + mappers
    │       │
    │       └── Bootstrap (startup game detection)
    │
    └── Server (endpoints + UI)
```

---

## Task List

### Phase 1: Foundation — Schema, Domain, Traits

#### Task 1: Migration v5 — add `name` to games, drop checkpoints
**Description:** Update `src/storage/db.rs` migration to v5. Add `name TEXT NOT NULL DEFAULT 'Unnamed'` to `games`. Drop `checkpoints` table and its index.

**Acceptance criteria:**
- [ ] `games` table has `name` column.
- [ ] `checkpoints` table no longer exists in schema.
- [ ] Migration increments `user_version` to 5.

**Verification:**
- [ ] `cargo test db_tests` passes.
- [ ] New DB file opens without errors.

**Dependencies:** None
**Files touched:** `src/storage/db.rs`
**Estimated scope:** Small

---

#### Task 2: Domain model — `Game`, update `DbGame`
**Description:** Add `Game` domain struct to `src/model/mod.rs`. Update `src/storage/models/game.rs` (`DbGame`) to include `name`. Remove `Checkpoint` from `src/model/mod.rs`.

**Acceptance criteria:**
- [ ] `Game` struct exists with `id`, `name`, `world_name`, `created_at`, `updated_at`.
- [ ] `DbGame` has `name: String`.
- [ ] `Checkpoint` no longer exported from `model` module.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 1
**Files touched:** `src/model/mod.rs`, `src/storage/models/game.rs`
**Estimated scope:** Small

---

#### Task 3: Storage traits — `set_game_id`, game CRUD, remove checkpoints
**Description:** Update `SnapshotStorage` trait in `src/storage/snapshot_storage.rs`: add `set_game_id`, add game methods (`list_games`, `create_game`, `delete_game`, `get_game`), remove all checkpoint methods. Update `MessageStorage` trait in `src/storage/message_storage.rs`: add `set_game_id`.

**Acceptance criteria:**
- [ ] `SnapshotStorage` has `set_game_id(&self, game_id: u64)`.
- [ ] `SnapshotStorage` has game CRUD methods returning `Result<...>`.
- [ ] `SnapshotStorage` has no checkpoint methods.
- [ ] `MessageStorage` has `set_game_id(&self, game_id: u64)`.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 2
**Files touched:** `src/storage/snapshot_storage.rs`, `src/storage/message_storage.rs`
**Estimated scope:** Medium

---

#### Task 4: `InMemoryGameStorage` — multi-game support, remove checkpoints
**Description:** Update `src/test_support/in_memory_storage.rs`: replace `_game_id: u64` with `Arc<AtomicU64>`, filter snapshots/messages by active game_id, implement `set_game_id`, remove checkpoint fields/methods.

**Acceptance criteria:**
- [ ] `InMemoryGameStorage` stores snapshots/messages per game_id.
- [ ] `set_game_id` switches the active filter.
- [ ] No checkpoint code remains.

**Verification:**
- [ ] `cargo test` in `test_support` passes.

**Dependencies:** Task 3
**Files touched:** `src/test_support/in_memory_storage.rs`, `src/test_support/in_memory_storage_tests.rs`
**Estimated scope:** Medium

---

### Checkpoint: Foundation
- [ ] `cargo test` passes (except checkpoint-related tests not yet removed).
- [ ] `cargo clippy` clean.
- [ ] `cargo fmt` clean.

---

### Phase 2: SQLite Implementation

#### Task 5: `SqliteGameStorage` — mutable game_id + game CRUD
**Description:** Update `src/storage/snapshot_storage.rs`: wrap `game_id` in `Arc<AtomicU64>`, implement `set_game_id`, implement game CRUD using `games` table queries. Remove checkpoint methods.

**Acceptance criteria:**
- [ ] `SqliteGameStorage` filters snapshots/messages by mutable game_id.
- [ ] `create_game` inserts into `games` with generated name.
- [ ] `list_games` returns all `games` rows ordered by `updated_at DESC`.
- [ ] `delete_game` cascades delete to snapshots/messages for that game_id.
- [ ] No checkpoint code remains.

**Verification:**
- [ ] `cargo test snapshot_storage_tests` passes.

**Dependencies:** Task 4
**Files touched:** `src/storage/snapshot_storage.rs`
**Estimated scope:** Medium

---

#### Task 6: Mappers — update game mapper, remove checkpoint mapper
**Description:** Update `src/storage/mappers/game.rs` (or create it) to map `DbGame` ↔ `Game`. Remove `src/storage/mappers/checkpoint.rs` and `src/storage/models/checkpoint.rs`. Update `src/storage/mappers/mod.rs` and `src/storage/models/mod.rs`.

**Acceptance criteria:**
- [ ] `DbGame` → `Game` mapping works.
- [ ] Checkpoint mapper/model files removed.
- [ ] Module exports updated.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 5
**Files touched:** `src/storage/mappers/game.rs`, `src/storage/mappers/mod.rs`, `src/storage/models/mod.rs`, `src/storage/mappers/checkpoint.rs` (delete), `src/storage/models/checkpoint.rs` (delete)
**Estimated scope:** Small

---

### Checkpoint: SQLite Layer
- [ ] All storage tests pass.
- [ ] `cargo clippy` clean.

---

### Phase 3: Bootstrap & Startup

#### Task 7: Game naming utility + query logic
**Description:** Add `generate_game_name(world_name, existing_names)` helper. Add startup query: "find most recent game for world X".

**Acceptance criteria:**
- [ ] `generate_game_name("Redmist", &["Redmist_2026-05-21_1"])` returns `"Redmist_2026-05-21_2"`.
- [ ] First gap is filled: `generate_game_name("Redmist", &["Redmist_2026-05-21_1", "Redmist_2026-05-21_3"])` returns `"Redmist_2026-05-21_2"`.

**Verification:**
- [ ] Unit tests for naming logic pass.

**Dependencies:** Task 6
**Files touched:** New file (e.g. `src/model/game.rs` or inline in `src/bootstrap/run.rs`)
**Estimated scope:** Small

---

#### Task 8: Update `bootstrap/run.rs` for multi-game startup
**Description:** After `DbPool` creation, query games for the selected world. If found, load the latest and use its `game_id`. If not found, create a new game with `generate_game_name`, then proceed with initial state save.

**Acceptance criteria:**
- [ ] Server startup with no games auto-creates a named game.
- [ ] Server startup with existing games loads the most recent one.
- [ ] Initial snapshot is saved into the correct game.

**Verification:**
- [ ] Manual: start server twice — first time creates `..._1`, second time loads it (does not create `..._2`).

**Dependencies:** Task 7
**Files touched:** `src/bootstrap/run.rs`
**Estimated scope:** Medium

---

### Checkpoint: Bootstrap
- [ ] Server starts cleanly with no existing DB.
- [ ] Server starts cleanly with existing DB containing games.

---

### Phase 4: Server & UI

#### Task 9: Add game routes to `server/mod.rs`
**Description:** Add routes: `GET /fragment/games`, `POST /games`, `POST /games/:id/switch`, `POST /games/:id/delete`. Remove checkpoint routes.

**Acceptance criteria:**
- [ ] Router includes all game endpoints.
- [ ] No checkpoint routes remain.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 8
**Files touched:** `src/server/mod.rs`
**Estimated scope:** Small

---

#### Task 10: Game fragment handlers
**Description:** Create `src/server/fragments/games.rs` with handlers: list games, create game, switch game, delete game. Switching game calls `set_game_id` on storage and triggers a full page reload or fragment refresh.

**Acceptance criteria:**
- [ ] `list_games_fragment` renders HTML with game names + switch/delete buttons.
- [ ] `create_game_handler` creates a new game with optional name (auto-name if empty).
- [ ] `switch_game_handler` changes active game_id and returns reload trigger.
- [ ] `delete_game_handler` removes game and cascades data.

**Verification:**
- [ ] Integration tests for endpoints pass.

**Dependencies:** Task 9
**Files touched:** `src/server/fragments/games.rs` (new), `src/server/fragments/mod.rs`
**Estimated scope:** Medium

---

#### Task 11: Update header template with game selector
**Description:** Update `src/server/templates.rs` `HeaderTemplate` to include a game selector dropdown showing the active game name and buttons to create/switch/delete.

**Acceptance criteria:**
- [ ] Header shows active game name.
- [ ] Dropdown/menu allows switching games.
- [ ] "New Game" button creates a game and reloads.

**Verification:**
- [ ] Screenshot or manual browser check confirms game UI is visible.

**Dependencies:** Task 10
**Files touched:** `src/server/templates.rs`, `assets/index.html` (if needed)
**Estimated scope:** Small

---

#### Task 12: Update reset handler to be game-scoped
**Description:** Update `src/server/fragments/misc.rs` `reset_handler` to clear only the current game's snapshots and messages (not all games). Use the current `game_id` from storage.

**Acceptance criteria:**
- [ ] Reset clears only active game data.
- [ ] Other games remain untouched.

**Verification:**
- [ ] Integration test: create two games, reset one, verify other still has data.

**Dependencies:** Task 10
**Files touched:** `src/server/fragments/misc.rs`
**Estimated scope:** Small

---

### Checkpoint: Server & UI
- [ ] All server tests pass.
- [ ] Manual browser check: can create, switch, delete games.
- [ ] Reset only affects active game.

---

### Phase 5: Remove Checkpoint Code

#### Task 13: Remove checkpoint server code
**Description:** Delete `src/server/fragments/checkpoint.rs`. Remove checkpoint re-exports from `src/server/fragments/mod.rs`. Remove checkpoint UI references from templates.

**Acceptance criteria:**
- [ ] `checkpoint.rs` file deleted.
- [ ] No checkpoint imports remain in server module.
- [ ] No checkpoint buttons in templates.

**Verification:**
- [ ] `cargo check` passes.

**Dependencies:** Task 11
**Files touched:** `src/server/fragments/checkpoint.rs` (delete), `src/server/fragments/mod.rs`, `src/server/templates.rs`
**Estimated scope:** Small

---

#### Task 14: Remove checkpoint model and mapper files
**Description:** Delete `src/model/checkpoint.rs`, `src/storage/models/checkpoint.rs`, `src/storage/mappers/checkpoint.rs`. Update any remaining imports.

**Acceptance criteria:**
- [ ] All checkpoint source files deleted.
- [ ] No remaining references to `Checkpoint` or `DbCheckpoint`.

**Verification:**
- [ ] `cargo check` passes.
- [ ] `grep -r "Checkpoint" src/` returns only false positives.

**Dependencies:** Task 13
**Files touched:** `src/model/checkpoint.rs` (delete), `src/storage/models/checkpoint.rs` (delete), `src/storage/mappers/checkpoint.rs` (delete)
**Estimated scope:** Small

---

### Checkpoint: No Checkpoints Remain
- [ ] `cargo test` passes.
- [ ] `cargo clippy` clean.
- [ ] No checkpoint code in the codebase.

---

### Phase 6: Documentation

#### Task 15: Update `data_layer.md`
**Description:** Update `docs/reference/data_layer.md` to document `games.name`, removed `checkpoints` table, and multi-game behaviour.

**Acceptance criteria:**
- [ ] `data_layer.md` reflects current schema.
- [ ] No checkpoint references.

**Verification:**
- [ ] Human review of doc changes.

**Dependencies:** Task 14
**Files touched:** `docs/reference/data_layer.md`
**Estimated scope:** Small

---

#### Task 16: Create `CONTEXT.md` and ADR
**Description:** Create `chronicler_engine/CONTEXT.md` with `Game` glossary entry. Create ADR for removing checkpoints in favour of multi-game support.

**Acceptance criteria:**
- [ ] `CONTEXT.md` exists with `Game` definition.
- [ ] ADR explains why checkpoints were removed and multi-game support introduced.

**Verification:**
- [ ] Human review.

**Dependencies:** Task 15
**Files touched:** `chronicler_engine/CONTEXT.md` (new), `docs/adr/adr-0XX-multi-game.md` (new)
**Estimated scope:** Small

---

### Phase 7: Final Validation

#### Task 17: Full test suite
**Description:** Run `cargo test` and fix any failures. Run `cargo clippy -- -D warnings` and fix lints. Run `cargo fmt`.

**Acceptance criteria:**
- [ ] `cargo test` passes 100%.
- [ ] `cargo clippy` clean.
- [ ] `cargo fmt` makes no changes.

**Verification:**
- [ ] CI-equivalent command passes.

**Dependencies:** Task 16
**Files touched:** Potentially any file touched above.
**Estimated scope:** Medium

---

#### Task 18: Build script validation
**Description:** Run `python build.py` from the `chronicler_engine/` directory to run the full validation pipeline.

**Acceptance criteria:**
- [ ] `python build.py` exits 0.

**Verification:**
- [ ] Build script output shows all green.

**Dependencies:** Task 17
**Files touched:** None (verification only)
**Estimated scope:** Small

---

### Checkpoint: Complete
- [ ] All tasks done.
- [ ] All tests pass.
- [ ] No checkpoint code remains.
- [ ] Documentation updated.
- [ ] Ready for review.

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `set_game_id` changes storage trait signatures, breaking many tests | Medium | Update `InMemoryGameStorage` in the same PR; compiler will flag all breakage. |
| Game switch requires reloading game state in AppState | Medium | Switch endpoint triggers a full page reload via HTMX rather than trying to hot-swap state. |
| Removing checkpoints may break tests that rely on checkpoint behaviour | Low | Those tests will fail compilation and be removed/updated as part of the plan. |
| Migration v5 drops checkpoints table — data loss for existing users | Low | Chronicler is pre-release; migration policy explicitly allows breaking changes. |

## Open Questions
- None — all clarified during grilling session.
