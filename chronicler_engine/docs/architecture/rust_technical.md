# Rust Implementation Idioms

Cross-cutting Rust conventions in this codebase. Point here instead of re-explaining the same idiom in every behavior or system doc.

## Scope rule

- **Entry:** recurring Rust convention (appears in 2+ places) with a statable reason for existing.
- **Not an entry:** localized Rust detail tied to one system's behavior. Stays inline in that system doc.
- **Do not restate** what an ADR or `guardrails.md` already covers. Cross-reference.

When in doubt about whether a Rust detail belongs in a behavior doc: if the fact doesn't change a reader's understanding of the system's *behavior*, it belongs here or nowhere.

## Idioms

### Sync services + no trait objects

`GameService` is a concrete struct, not a trait. `Storage` is a concrete struct backed by a `Backend` enum (`Sqlite` / `InMemory`) plus a `BackendKind` decorator (`Direct` / `Test`), not a trait-based repository. No `dyn Trait`, `Arc<dyn>`, or `#[async_trait]` in the application or storage layers.

Why: avoids `#[async_trait]` + `dyn Trait` incompatibility in Rust 2024 edition; removes trait boilerplate and custom mocks. Async I/O is pushed to the HTTP layer via `spawn_blocking` (see below) rather than threaded through the application as async traits.

### `spawn_blocking` offload

Synchronous services (`GameService`, `ActionPipeline`) run inside `tokio::task::spawn_blocking` from HTTP handlers in `src/adapters/driving/http/fragments/actions.rs`. The pipeline instance is constructed once at startup and shared by `Arc`.

Why: prevents the Axum event loop from stalling during LLM network I/O while keeping application code synchronous. Not architecturally interesting on its own — don't re-explain in every system doc that touches the pipeline or LLM path.

### `Arc<RwLock<AppSettings>>` settings sharing

Settings loaded once at bootstrap (`bootstrap/run.rs`), wrapped in `Arc<RwLock<AppSettings>>`, and passed through the construction chain. No business logic layer reloads settings from disk. Connection changes require a server restart; only `max_context_tokens` is read dynamically at runtime.

Why: settings are read-mostly and shared across services. `RwLock` permits concurrent reads; rare writes (settings reload) are deliberate.

See `system/startup.md`, `system/llm_processing.md`.

### Poison recovery via `into_inner()`

All `Mutex` / `RwLock` sites recover from poison by calling `into_inner()` on the poisoned lock. No panic propagation from lock poisoning.

See `architecture/guardrails.md` (§3, with test reference `tests/poison_recovery.rs`).

### `Arc<AtomicBool>` as cached projection

`is_generating` is an `Arc<AtomicBool>` on `DefaultApplicationService` that is a read-only cache of the persisted per-game registry state (`Arc<RwLock<HashMap<GameId, GenerationSlot>>>`). The registry is write-side truth; the atomic is a read-only projection of "any slot Generating".

Why: read-mostly hot path. Avoids lock contention for the common polling query.

See `architecture/system.md` §Tier Map.

### `CancellationToken` shutdown gate

`is_shutting_down()` (backed by `AppState.shutdown_token`) is checked at the HTTP entry boundary only — never inside phase functions. Long-running pipelines perform an in-phase game-id α-check (`PipelineRun::check_game_unchanged(started_for)`) at three internal boundaries; this is *behavioral*, documented in `system/action_pipeline.md` §Cancellation, not here.

Why: shutdown signal is a runtime concern at the driving adapter, not a phase concern. Keeping the check out of phase functions preserves phase purity (phases operate on `GameState`, not on runtime signals).

## Reserved

(Add new entries only when a Rust idiom recurs across 2+ docs and the existing inline mentions are noise rather than load-bearing.)

## Document References

- [ADR-030: `is_generating` Invariant](../adr/adr-030-is-generating-invariant.md) — registry vs atomic, ownership check, projection update rationale
