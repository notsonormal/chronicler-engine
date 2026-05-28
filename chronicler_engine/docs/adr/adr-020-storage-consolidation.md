# ADR-020: Unified Storage Struct

## Status

**Accepted**

## Context

The storage layer had grown to six separate traits (`GameStorage`, `SnapshotStorage`, `MessageStorage`, `MessageSwipeStorage`, `PromptPresetStorage`, `LlmMessageStorage`) with 12 repository structs (SQLite + in-memory pairs). This produced:

- **1,371 lines** across `src/storage/*_storage.rs` and `src/test_support/in_memory_storage.rs`
- **Six `Arc<dyn Trait>` fields** on `GameServiceContext` (later five after ADR-019)
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
4. **Table-scoped methods preserved.** No `Storage` method touches more than one table; cross-table coordination stays in `GameServiceContext` helpers (`load_messages_with_swipes`, `save_message_and_snapshot`, etc.). This preserves ADR-019's intent without the module overhead.

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

## Migration

1. Create `Storage` struct with all methods from the six traits.
2. Update `GameServiceContext`, `AppState`, `ServerResources` to use `Arc<Storage>`.
3. Update bootstrap to construct one `Storage::new_sqlite()` instead of five repositories.
4. Rewrite all test doubles to use `Backend::Test` + `TestFailureHandle`.
5. Delete old trait files, repository structs, and in-memory implementations.
6. Remove dead `guardrails_one_table_per_storage` guardrail (ADR-019 is superseded).

## References

- `src/storage/backend.rs` — Unified `Storage` struct
- `src/application/context.rs` — `GameServiceContext` and cross-storage helpers
- ADR-019 — Previous "one table per storage module" decision (now superseded)
- `booster-gold-damage-domino.md` — Implementation plan
