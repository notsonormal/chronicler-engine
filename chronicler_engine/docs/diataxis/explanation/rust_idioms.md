---
diataxis: explanation
title: Rust Idioms
---

## Concrete services and backend enum dispatch

`GameService` is a concrete struct. So is `Storage`. `Storage` is backed by a `Backend` enum — `Sqlite` or `InMemory` — plus a `BackendKind` decorator (`Direct` or `Test`). The application and storage layers consist of concrete types composed into the construction chain; trait objects are reserved for places where the polymorphism they enable is the point (the agent registry's `Box<dyn Agent>` for heterogeneous agent dispatch).

Backend switching travels through enum-variant dispatch. Tests construct an in-memory backend by selecting the enum variant; production constructs the SQLite backend the same way; the surrounding code is the same struct. The cost sits in the small switch sites that consume the enum; the saving is that every call site reads the same concrete type rather than threading a type parameter through the application to make storage polymorphic.

The convention fits the constraint that Rust 2024 edition makes async-trait dispatch awkward to compose with synchronous backend code. Pushing async I/O to the HTTP layer via `spawn_blocking` (next section) keeps the application code synchronous and lets backend types stay concrete, which avoids the trait boilerplate and per-test custom mocks that a trait repository would carry.

## LLM-call offload via spawn_blocking

Synchronous services (`GameService`, `ActionPipeline`) run inside `tokio::task::spawn_blocking`. The spawn helper lives at `src/application/utils/spawn.rs`; HTTP handlers reach it through `DefaultApplicationService::process_action` and `GenerationGate::start_action` rather than calling it directly. The pipeline instance is built once at startup and shared through an `Arc`, so the handler submits work to the same pipeline across requests.

The offload buys separation between the Axum event loop, which stays responsive, and the LLM network call, which can take seconds. The synchronous service code is unchanged; the handler hands the blocking call to a Tokio blocking pool, returns immediately, and the caller awaits the response on the future the pool returns. The architectural cost is one allocation per request; the operational gain is that latency from one slow LLM call does not back up unrelated handlers.

## Settings sharing through `Arc<RwLock<AppSettings>>`

`AppSettings` is loaded once at bootstrap (`bootstrap/run.rs`), wrapped in `Arc<RwLock<AppSettings>>`, and passed through the construction chain to every component that needs to read it. No business logic layer reloads from disk. Connection changes — model swap, endpoint change — require a server restart; only `max_context_tokens` is read dynamically on each LLM call, with the read taken at the call site.

The shape serves a read-mostly workload. Many components hold the settings handle and read concurrently; the `RwLock` permits parallel reads and serialises the rare writes (settings-mutating handlers). Components that need to apply a settings change acquire the write lock, mutate, release; components that just need current values acquire the read lock briefly and copy what they need. The cost — one allocation per settings read — is dwarfed by the LLM call the read accompanies.

## Lock-poison recovery

Every `Mutex` and `RwLock` site in the engine recovers from a poisoned lock by calling `.into_inner()` on the guard. Lock poisoning is the Rust standard library's signal that a previous holder panicked while holding the lock; the engine treats poisoning as recoverable rather than fatal.

The recovery is the recovery path. A panic while holding the lock is a definite bug — the previous code path exited abnormally — but the engine's policy is that the next holder still gets access to the data the lock protects. `.into_inner()` consumes the poisoned `Result` and yields the inner value; the type is still the same; the next holder sees the state as the previous holder left it and proceeds. The convention holds at every site with a single consistent shape. The invariant is asserted in `tests/poison_recovery.rs`.

## Atomic projection of the generation registry

`is_generating` is an `Arc<AtomicBool>` on `DefaultApplicationService` — a read-only cache of the persisted per-game generation registry (`Arc<RwLock<HashMap<GameId, GenerationSlot>>>`, where a `GenerationSlot` carries the generation state for one game). The registry is the write-side truth; the atomic is a projection the hot path reads without taking the registry's lock.

The projection serves the HTTP poll path, which is the highest-frequency reader in the engine. The UI polls `/is-generating` to decide whether to show a spinner; the lock-free atomic read keeps that request off the registry's `RwLock` for the common "not generating" case. The atomic is updated whenever the registry mutates — the projection is owned by the same code that owns the write, so it cannot drift. Ownership invariants for the atomic and the registry are detailed in the `is_generating` ADR.

## Shutdown gate at the spawn boundary

Shutdown is signalled through a `CancellationToken` on `AppState`. The token is read at the application layer rather than at the HTTP handler entry. Retry and retrigger check the token twice: a prepare-state helper captures the snapshot and reads the token before the spawn (the caller skips the spawn on shutdown), and the spawn_blocking closure reads the token again before the pipeline runs. The process-action path checks inside the spawn_blocking closure only, at the entry of the spawned task before the pipeline runs. The check does not live inside phase functions; long-running pipelines perform an in-phase game-id α-check (`PipelineRun::check_game_unchanged(started_for)`) at three internal boundaries instead, which is *behavioural* rather than runtime.

The boundary reading keeps phase functions pure: phases operate on `GameState`, not on runtime signals. The shutdown check is a runtime concern at the driving adapter, not a domain concern in the pipeline. The α-check inside the pipeline covers the case where the request started before shutdown was signalled but the LLM call is still in flight when the signal arrives — the pipeline notices that the game id it was started for no longer matches the registry, and bails. The two checks together cover request-arrival and pipeline-in-flight; nothing needs to be cancelled from inside a phase.

## Document References

- [ADR-010: Concurrency Generation Gate](../../docs/adr/adr-010-concurrency-generation-gate.md) — `spawn_blocking` offload; sync services over async traits; the runtime concurrency frame the engine sits inside.
- [ADR-030: `is_generating` Invariant](../../docs/adr/adr-030-is-generating-invariant.md) — registry vs atomic, ownership check, projection update rationale.
- `../reference/architecture_system.md` — tier map (the canonical home for the `Arc<RwLock<AppSettings>>` and `Arc<AtomicBool>` shapes) + invariant identifiers.
- `../reference/guardrails.md` — INV-NNN *identifiers* (the guarantee for each lives in the invariant contract tests, not the docs).
- `../reference/llm_processing.md` — LLM transport + the per-call site that reads `max_context_tokens` from settings.
- `../reference/action_pipeline.md` — pipeline cancellation shape (the in-phase α-check that lives inside the pipeline rather than at the boundary).
- `../explanation/architecture.md` [§Quality Story](../explanation/architecture.md#quality-story) — the quality attributes the conventions above guarantee.
