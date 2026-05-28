# Revised Plan: Consolidate Storage Trait System

## Overview

Replace 6 storage traits + 12 repository structs (~1,371 lines) with a single concrete `Storage` struct backed by an enum. All `Arc<dyn Trait>` injection points become `Arc<Storage>`.

**Target:** Reduce storage-related source code by >50% while preserving all test patterns (including dynamic failure injection).

## Architecture Decisions

1. **Single `Storage` struct, not a trait.** The codebase has exactly 2 real backends (SQLite for prod, HashMap for tests). Traits are overkill for this multiplicity.
2. **Shared `game_id` on `Storage`, not per-backend.** `GameServiceContext::set_game_id()` currently calls `set_game_id` on 3 separate repositories. A single `AtomicU64` on `Storage` eliminates duplication and prevents divergence bugs.
3. **Test failure injection via `Backend::Test` variant.** Uses `Arc<Mutex<HashMap<Operation, TestOverride>>>` so tests can inject failures statically or dynamically (toggle mid-test).
4. **Preserve ADR-019's intent, not its mechanism.** ADR-019 wanted to prevent hidden cross-table transactions in repositories. The unified `Storage` struct keeps methods table-scoped (no method touches more than one table) but colocates them in one type.

## Design Sketch

```rust
pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<Backend>,
}

enum Backend {
    Sqlite { pool: DbPool },
    InMemory {
        snapshots: HashMap<u64, Vec<GameStateSnapshot>>,
        next_snapshot_id: u64,
        games: Vec<Game>,
        next_game_id: u64,
        messages: HashMap<u64, Vec<Message>>,
        next_message_id: u64,
        swipes: HashMap<u64, Vec<Swipe>>,
        presets: Vec<PromptPreset>,
        llm_messages: Vec<LlmMessage>,
    },
    Test {
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<Operation, TestOverride>>>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    SaveSnapshot,
    LoadLatestSnapshot,
    LoadSnapshotById,
    ListGames,
    CreateGame,
    DeleteGame,
    GetGame,
    InsertMessage,
    DeleteMessage,
    LoadMessageRows,
    GetActiveSwipeIndex,
    UpdateActiveSwipe,
    SoftDeleteMessage,
    RestoreSoftDeleted,
    PurgeSoftDeleted,
    InsertSwipe,
    UpdateSwipeText,
    ShiftSwipeIndices,
    LoadSwipesForMessages,
    ListPresets,
    GetPreset,
    SavePreset,
    DeletePreset,
    SaveLlmMessage,
    ListLatestLlmMessages,
}

pub struct TestOverride {
    kind: ErrorKind,
    message: String,
}

pub enum ErrorKind { Config, Internal }
```

## Task List

### Phase 1: Foundation

#### Task 1: Create `Storage` struct shell and `Backend` enum
**Description:** Define `Storage`, `Backend`, `Operation`, `TestOverride`, and `ErrorKind` in a new `src/storage/storage.rs`. Add constructors `Storage::new_sqlite(pool, game_id)` and `Storage::new_in_memory()`. Expose from `src/storage/mod.rs`.

**Acceptance criteria:**
- [ ] File compiles with `cargo check`
- [ ] `Storage` is `Send + Sync`
- [ ] Constructors exist and are usable

**Verification:**
- [ ] `cargo check` passes

**Dependencies:** None
**Files touched:** `src/storage/storage.rs` (new), `src/storage/mod.rs`
**Estimated scope:** Small

#### Task 2: Port GameStorage + SnapshotStorage methods
**Description:** Move `GameStorage` methods (`set_game_id`, `list_games`, `create_game`, `delete_game`, `get_game`) and `SnapshotStorage` methods (`save`, `load_latest`, `load_by_id`) into inherent `impl Storage` blocks. SQLite logic matches on `Backend::Sqlite`; in-memory logic matches on `Backend::InMemory`.

**Acceptance criteria:**
- [ ] All 8 methods exist on `Storage`
- [ ] SQLite paths use `self.game_id()` for game scoping
- [ ] In-memory paths replicate existing behavior (HashMap per game_id for snapshots, Vec for games)

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run game_storage_tests snapshot_storage_tests` passes (tests will still use old traits — this is OK for now)

**Dependencies:** Task 1
**Files touched:** `src/storage/storage.rs`
**Estimated scope:** Medium

#### Task 3: Port MessageStorage + MessageSwipeStorage methods
**Description:** Move `MessageStorage` methods (`insert_message`, `delete_message`, `load_message_rows`, `get_active_swipe_index`, `update_active_swipe`, `soft_delete_message`, `restore_soft_deleted`, `purge_soft_deleted`) and `MessageSwipeStorage` methods (`insert_swipe`, `update_swipe_text`, `shift_swipe_indices`, `load_swipes_for_messages`) into `impl Storage`.

**Acceptance criteria:**
- [ ] All 12 methods exist on `Storage`
- [ ] SQLite paths filter by `game_id`
- [ ] In-memory paths replicate existing behavior

**Verification:**
- [ ] `cargo check` passes

**Dependencies:** Task 1
**Files touched:** `src/storage/storage.rs`
**Estimated scope:** Medium

#### Task 4: Port PromptPresetStorage + LlmMessageStorage methods
**Description:** Move `PromptPresetStorage` methods (`list`, `get`, `save`, `delete`) and `LlmMessageStorage` methods (`save`, `list_latest`) into `impl Storage`.

**Acceptance criteria:**
- [ ] All 6 methods exist on `Storage`
- [ ] SQLite paths work without game_id (global tables)
- [ ] In-memory paths replicate existing behavior

**Verification:**
- [ ] `cargo check` passes

**Dependencies:** Task 1
**Files touched:** `src/storage/storage.rs`
**Estimated scope:** Medium

#### Task 5: Add `Backend::Test` with failure injection
**Description:** Implement `Backend::Test` variant that delegates to `base` unless the operation is in `overrides`. Add helper constructors: `Storage::with_failure(self, op, override_)` and `Storage::with_shared_overrides(self, arc)` for dynamic toggle tests.

**Acceptance criteria:**
- [ ] `Test` variant intercepts overridden operations
- [ ] Non-overridden operations delegate to `base`
- [ ] `with_failure` returns `Self` for chaining
- [ ] `with_shared_overrides` accepts `Arc<Mutex<HashMap<Operation, TestOverride>>>`

**Verification:**
- [ ] `cargo check` passes
- [ ] Write a quick unit test in `src/storage/storage_tests.rs` verifying interception

**Dependencies:** Tasks 2-4
**Files touched:** `src/storage/storage.rs`, `src/storage/storage_tests.rs` (new)
**Estimated scope:** Small

### Checkpoint: After Phase 1
- [ ] `Storage` struct is complete with all methods
- [ ] `cargo check` passes
- [ ] Unit test for `Backend::Test` passes

### Phase 2: Integration

#### Task 6: Update `GameServiceContext`
**Description:** Change `GameServiceContext` fields from 5 `Arc<dyn Trait>` to 5 `Arc<Storage>` (or fewer — see note). Update `set_game_id()` to call `storage.set_game_id()` once. Update `load_messages()`, `update_message_text()`, and cross-storage helpers to use `&Storage`.

**Note:** We could collapse the 5 storage fields into a single `storage: Arc<Storage>`. Evaluate during implementation whether this simplifies `GameServiceContext` further. The plan allows either 1 or 5 fields — whichever produces cleaner code.

**Acceptance criteria:**
- [ ] `GameServiceContext` compiles
- [ ] `set_game_id` is a single call
- [ ] `load_messages_with_swipes` signature updated if needed

**Verification:**
- [ ] `cargo check` passes

**Dependencies:** Tasks 1-5
**Files touched:** `src/application/context.rs`
**Estimated scope:** Medium

#### Task 7: Update `AppState`, `ServerResources`, and bootstrap
**Description:** Change `AppState` and `ServerResources` storage fields to `Arc<Storage>`. Update `bootstrap/run.rs` to construct a single `Storage::new_sqlite(pool, game_id)` instead of 5 separate repositories. Update `as_game_service_context()`.

**Acceptance criteria:**
- [ ] `AppState` and `ServerResources` compile
- [ ] `bootstrap/run.rs` constructs one `Storage`
- [ ] Server startup logic (`load_latest`, `load_messages_with_swipes`) uses `Storage` methods

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run bootstrap::run_tests` passes

**Dependencies:** Task 6
**Files touched:** `src/server/mod.rs`, `src/bootstrap/run.rs`
**Estimated scope:** Medium

#### Task 8: Update `test_support/context.rs`
**Description:** Change `make_test_context`, `make_test_context_without_snapshot`, and `make_test_context_with_sqlite` to construct `Arc<Storage>` instead of separate repositories. Update `build_test_context` signature.

**Acceptance criteria:**
- [ ] All 3 test context helpers compile
- [ ] In-memory variant used for `make_test_context` and `make_test_context_without_snapshot`
- [ ] SQLite variant used for `make_test_context_with_sqlite`

**Verification:**
- [ ] `cargo check --tests` passes

**Dependencies:** Task 6
**Files touched:** `src/test_support/context.rs`
**Estimated scope:** Small

### Checkpoint: After Phase 2
- [ ] `cargo check --all-targets` passes
- [ ] All context, bootstrap, and server code compiles

### Phase 3: Test Migration

#### Task 9: Migrate storage unit tests
**Description:** Move existing storage tests from `src/storage/*_storage_tests.rs` and `src/test_support/in_memory_storage_tests.rs` to `src/storage/storage_tests.rs`. Update them to use `Storage` instead of traits. Delete old test files.

**Acceptance criteria:**
- [ ] All storage tests run against `Storage` methods
- [ ] Both SQLite and in-memory paths tested
- [ ] Old test files deleted

**Verification:**
- [ ] `cargo nextest run storage_tests` passes

**Dependencies:** Tasks 1-8
**Files touched:** `src/storage/storage_tests.rs`, delete `src/storage/*_storage_tests.rs`, delete `src/test_support/in_memory_storage_tests.rs`
**Estimated scope:** Medium

#### Task 10: Migrate failing test doubles (context_tests.rs)
**Description:** Replace `FailingSnapshotStorage` and `FailingPresetStorage` in `src/application/context_tests.rs` with `Storage::in_memory().with_failure(...)`.

**Acceptance criteria:**
- [ ] `FailingSnapshotStorage` struct deleted
- [ ] `FailingPresetStorage` struct deleted
- [ ] Tests compile and pass

**Verification:**
- [ ] `cargo nextest run application::context_tests` passes

**Dependencies:** Tasks 1-8
**Files touched:** `src/application/context_tests.rs`
**Estimated scope:** Small

#### Task 11: Migrate failing test doubles (pipeline_tests.rs + retry_tests.rs)
**Description:** Replace `FailingSaveStorage` in pipeline_tests.rs and `FailingSnapshotStorage` + `FailingMessageStorage` in retry_tests.rs with `Storage` using `with_shared_overrides` for dynamic toggle tests.

**Acceptance criteria:**
- [ ] `FailingSaveStorage` deleted
- [ ] `FailingSnapshotStorage` (retry_tests.rs) deleted
- [ ] `FailingMessageStorage` deleted
- [ ] Dynamic toggle tests still work (set failure before call, clear after)

**Verification:**
- [ ] `cargo nextest run action_pipeline::pipeline_tests` passes
- [ ] `cargo nextest run action_pipeline::retry_tests` passes

**Dependencies:** Tasks 1-8
**Files touched:** `src/application/action_pipeline/pipeline_tests.rs`, `src/application/action_pipeline/retry_tests.rs`
**Estimated scope:** Medium

#### Task 12: Migrate failing test doubles (handlers_tests.rs)
**Description:** Replace `FailingPromptPresetStorage` in `src/server/prompt_presets_fragment/handlers_tests.rs` with `Storage::in_memory().with_shared_overrides(...)`.

**Acceptance criteria:**
- [ ] `FailingPromptPresetStorage` deleted
- [ ] Tests compile and pass

**Verification:**
- [ ] `cargo nextest run prompt_presets_fragment::handlers_tests` passes

**Dependencies:** Tasks 1-8
**Files touched:** `src/server/prompt_presets_fragment/handlers_tests.rs`
**Estimated scope:** Small

### Checkpoint: After Phase 3
- [ ] All tests pass: `cargo nextest run`
- [ ] No failing test double structs remain in codebase

### Phase 4: Cleanup & Validation

#### Task 13: Delete old trait files and update `storage/mod.rs`
**Description:** Delete the 6 trait files (`game_storage.rs`, `snapshot_storage.rs`, `message_storage.rs`, `message_swipe_storage.rs`, `prompt_preset_storage.rs`, `llm_message_storage.rs`) and `in_memory_storage.rs`. Update `storage/mod.rs` to only export `storage`, `db`, `models`, `mappers`.

**Acceptance criteria:**
- [ ] All 7 old files deleted
- [ ] `storage/mod.rs` only exports new modules
- [ ] No dead code warnings

**Verification:**
- [ ] `cargo check --all-targets` passes
- [ ] `cargo clippy --all-targets` passes

**Dependencies:** Tasks 9-12
**Files touched:** `src/storage/mod.rs`, delete 7 files
**Estimated scope:** Small

#### Task 14: Update architecture docs
**Description:** Update `docs/architecture/system.md` §7 to describe the unified `Storage` struct. Update `docs/reference/data_layer.md` code mapping section. Add `docs/adr/adr-020-storage-consolidation.md` recording the decision.

**Acceptance criteria:**
- [ ] `system.md` reflects single `Storage` struct
- [ ] `data_layer.md` updated
- [ ] ADR-020 created with context, decision, consequences

**Verification:**
- [ ] `python scripts/generate_docs_index.py` runs without error

**Dependencies:** Task 13
**Files touched:** `docs/architecture/system.md`, `docs/reference/data_layer.md`, `docs/adr/adr-020-storage-consolidation.md`
**Estimated scope:** Small

#### Task 15: Final validation
**Description:** Run full validation suite.

**Acceptance criteria:**
- [ ] `cargo fmt` makes no changes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo nextest run` passes
- [ ] `python build.py` passes
- [ ] Line count in `src/storage/` + `src/test_support/in_memory_storage.rs` reduced by >50%

**Verification:**
- [ ] All of the above

**Dependencies:** Tasks 13-14
**Files touched:** None (validation only)
**Estimated scope:** Small

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `cargo nextest run` fails due to subtle in-memory behavior difference | High | Port in-memory logic verbatim first, optimize later. Run tests after each phase. |
| Dynamic failure injection tests break | Medium | `Backend::Test` uses `Arc<Mutex<...>>` shared with test code, preserving toggle pattern. |
| `Storage` struct becomes too large | Low | All methods are table-scoped; no cross-table logic. File size target: <600 lines (still < half of current). |
| `EngineError` not `Clone` blocks `TestOverride` | Low | `TestOverride` stores `ErrorKind + String`, not `EngineError` directly. |
| Missing a trait usage site | Medium | Search for `dyn.*Storage` and `impl.*Storage` before deletion. Grep for each trait name. |

## Open Questions

- Should `GameServiceContext` collapse to a single `storage: Arc<Storage>` field, or keep separate fields for clarity? Evaluate during Task 6.
- Should `Operation` enum live in `storage.rs` or a sub-module? Prefer `storage.rs` unless it exceeds ~30 variants.
