# ADR-020: Unified Storage Struct

**Date:** 2026-05-28
**Status:** Accepted

> **Note (2026-06-27):** As of the Backend/LayeredBackend split refactor, `Backend::Test` was reorganized into `LayeredBackend::Test`. The original decision rationale stands — failure injection still uses `Arc<Mutex<HashMap<&'static str, TestOverride>>>` maps for both static setup (`with_failure`) and dynamic toggling (`TestFailureHandle::set` / `clear`). Type location changed (test-infra types moved to `storage::backend::test_support`); purpose unchanged.

## Context

The storage layer had grown to six separate traits (`GameStorage`, `SnapshotStorage`, `MessageStorage`, `MessageSwipeStorage`, `PromptPresetStorage`, `LlmMessageStorage`) with 12 repository structs (SQLite + in-memory pairs). This produced:

- **1,371 lines** across `src/storage/*_storage.rs` and `src/test_support/in_memory_storage.rs`
- **Six `Arc<dyn Trait>` fields** on `GameServiceContext` (later five after the one-table-per-module refactor that ADR-020 absorbs)
- **Six custom mock structs** for failure injection in tests (`FailingSnapshotStorage`, `FailingMessageStorage`, etc.)
- **Trait boilerplate** for a domain that has exactly two real backends (SQLite for production, HashMap for tests)

The codebase had only two backends, yet paid the full cost of trait object indirection — vtables, `Arc<dyn>` cloning, and per-trait mock implementations.

## Decision

**Replace all storage traits with a single concrete `Storage` struct backed by a `Backend` enum.**

```rust
pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<Backend>,
}

enum Backend {
    Sqlite { pool: DbPool },
    InMemory(Box<InMemoryData>),
    Test {
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<Operation, TestOverride>>>,
    },
}
```

All previous trait methods become inherent methods on `Storage`. Callers pass `Arc<Storage>` instead of `Arc<dyn Trait>`.

### Key Design Choices

1. **Enum over trait.** Two real backends → enum is simpler, faster (no vtables), and enables exhaustive matching.
2. **Shared `game_id` on `Storage`.** One `AtomicU64` eliminates the duplication of `set_game_id` across five repositories and prevents divergence bugs.
3. **`Backend::Test` for failure injection.** `Arc<Mutex<HashMap<Operation, TestOverride>>>` allows static setup (`with_failure`) and dynamic toggling (`TestFailureHandle::set` / `clear`) mid-test. This replaces all custom failing mock structs.
4. **Table-scoped methods preserved.** No `Storage` method touches more than one table; cross-table coordination stays in `GameServiceContext` helpers (`load_messages_with_swipes`, `save_message_and_snapshot`, etc.). This preserves the one-table-per-module intent without the module overhead.

## Consequences

### Positive

- **~55% reduction** in storage-related source code (1,371 lines → ~620 lines)
- **Zero trait boilerplate** — no `dyn Trait`, no `Arc<dyn>`, no per-trait mocks
- **Single `Arc<Storage>` field** on `GameServiceContext` instead of five `Arc<dyn>` fields
- **Dynamic failure injection** without custom structs — `Storage::new_in_memory().with_test_failures()` returns a `(Storage, TestFailureHandle)` pair
- **Exhaustive matching** on `Backend` — compiler catches unhandled variants

### Negative

- **`Storage` is a large type.** All table methods live on one struct. The file is ~1,200 lines (still under the 2,000-line guardrail). If it grows further, split by table into `impl Storage` blocks in the same file.
- **No trait-based polymorphism.** If a third backend (e.g. PostgreSQL) is added, the enum grows. This is acceptable because the project has no roadmap for additional backends.
- **`Backend` enum size.** `InMemory` variant is large; boxing it (`Box<InMemoryData>`) keeps the enum small for the production `Sqlite` path.

### Trade-offs
- Chose single concrete struct over trait-based polymorphism (simplicity won; no roadmap for third backend)
- Chose enum dispatch over dyn Trait (compiler-exhaustive matching won over open extensibility)
- Chose dynamic failure injection over per-trait mock structs (test ergonomics won; enum size mitigated with `Box`)

