# ADR-020: Unified Storage Struct

**Date:** 2026-05-28
**Status:** Accepted

## Context

The storage layer had grown to six separate traits (`GameStorage`, `SnapshotStorage`, `MessageStorage`, `MessageSwipeStorage`, `PromptPresetStorage`, `LlmMessageStorage`) with 12 repository structs (SQLite + in-memory pairs). This produced:

- **Six `Arc<dyn Trait>` fields** on `GameServiceContext`
- **Six custom mock structs** for failure injection in tests (`FailingSnapshotStorage`, `FailingMessageStorage`, etc.)
- **Trait boilerplate** for a domain that has exactly two real backends (SQLite for production, HashMap for tests)

The codebase had only two backends, yet paid the full cost of trait object indirection — vtables, `Arc<dyn>` cloning, and per-trait mock implementations.

## Decision

**Replace all storage traits with a single concrete `Storage` struct backed by `Backend` and `LayeredBackend` enums.**

```rust
pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<LayeredBackend>,
}

enum Backend {
    Sqlite { pool: DbPool },
    InMemory(Box<InMemoryData>),
}

enum LayeredBackend {
    Direct(Backend),
    #[cfg(feature = "testing")]
    Test {
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>>,
    },
}
```

`LayeredBackend::Test.base` is `Box<Backend>` (not `Box<LayeredBackend>`) — a non-recursive design that enforces at most one Test layer per storage instance (replace-not-nest).

All previous trait methods become inherent methods on `Storage`. Callers pass `Arc<Storage>` instead of `Arc<dyn Trait>`.

### Key Design Choices

1. **Enum over trait.** Two real backends → enum is simpler, faster (no vtables), and enables exhaustive matching.
2. **Shared `game_id` on `Storage`.** One `AtomicU64` eliminates the duplication of `set_game_id` across five repositories and prevents divergence bugs.
3. **`LayeredBackend::Test` for failure injection.** `Arc<Mutex<HashMap<&'static str, TestOverride>>>` allows static setup (`with_failure`) and dynamic toggling (`TestFailureHandle::set` / `clear`) mid-test. This replaces all custom failing mock structs.
4. **Table-scoped methods preserved.** No `Storage` method touches more than one table; cross-table coordination stays in `GameServiceContext` helpers (`load_messages_with_swipes`, `save_message_and_snapshot`, etc.). This preserves the one-table-per-module intent without the module overhead.

## Consequences

### Positive

- **Zero trait boilerplate** — no `dyn Trait`, no `Arc<dyn>`, no per-trait mocks
- **Replaces five `Arc<dyn>` repository fields** on `GameServiceContext` with `Arc<Storage>` (plus a second `Arc<Storage>` for presets)
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

## History

- **2026-06-27**: `Backend::Test` reorganised into `LayeredBackend::Test` as part of the Backend/LayeredBackend split refactor. Original decision rationale stands — failure injection still uses `Arc<Mutex<HashMap<&'static str, TestOverride>>>` maps for static setup (`with_failure`) and dynamic toggling (`TestFailureHandle::set` / `clear`). Test-infra types moved to `storage::backend::test_support`; purpose unchanged.

