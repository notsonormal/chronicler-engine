---
diataxis: explanation
title: Rust Idioms
---

## Concrete services and backend enum dispatch

`Job`, `GameWriter` and the catalogue collaborators are concrete structs. So is `Storage`. `Storage` is backed by a `Backend` enum — `Sqlite` or `InMemory` — plus a `BackendKind` decorator (`Direct` or `Test`). The application and storage layers consist of concrete types composed into the construction chain; trait objects are reserved for places where the polymorphism they enable is the point (the `AgentRunner`'s `Box<dyn AnalysisAgent>` for heterogeneous agent dispatch).

Backend switching travels through enum-variant dispatch. Tests construct an in-memory backend by selecting the enum variant; production constructs the SQLite backend the same way; the surrounding code is the same struct. Each call site reads the same concrete type; the polymorphism lives at the switch sites that consume the enum, which avoids threading a type parameter through the application to make storage polymorphic.

The convention fits the constraint that Rust 2024 edition makes async-trait dispatch awkward to compose with synchronous backend code. Pushing async I/O to the HTTP layer via `spawn_blocking` (next section) keeps the application code synchronous and lets backend types stay concrete, which avoids the trait boilerplate and per-test custom mocks that a trait repository would carry.

## LLM-call offload via spawn_blocking

Synchronous services (`Job` and its collaborators) run inside `tokio::task::spawn_blocking`. `Job::start` accepts the request and spawns the Job on the blocking pool. HTTP handlers reach it through the `game_writer` and `job_deps` fields of `AppState`. The `JobDeps` are built once at startup and shared through `Arc`s, so each handler starts its Job on the same narrator, Agent runner and storage.

The offload buys separation between the Axum event loop, which stays responsive, and the LLM network call, which can take seconds. The handler gets its answer when the accept succeeds, and the Job runs on after the response. Latency from one slow LLM call does not back up unrelated handlers; the synchronous service code runs unchanged inside the blocking pool.

## Settings resolution at the read site

`AppSettings` lives as a singleton row in the database. Components that need settings call `Storage::get_settings` at the point of use and pass the value down the call; nothing holds a long-lived settings handle. A settings-mutating handler goes through `Storage::update_settings`, which performs the read, the mutation, and the write under one storage acquisition, so two overlapping edits cannot clobber each other.

The shape serves a read-mostly workload. Each read is a short storage call; writes are the rare dashboard requests. Because the read sits beside the code that uses the value, a connection edit takes effect on the next LLM call — the in-flight call finishes on the provider it already resolved, and the following call picks up the new one.

## Lock-poison recovery

Every `Mutex` site in the engine recovers from a poisoned lock by calling `.into_inner()` on the guard. Lock poisoning is the Rust standard library's signal that a previous holder panicked while holding the lock; the engine treats poisoning as recoverable rather than fatal.

A panic while holding the lock is a definite bug — the previous code path exited abnormally — but the engine treats the next holder as legitimate. `.into_inner()` consumes the poisoned `Result` and yields the guarded value; the type is unchanged; the next holder sees the state as the previous holder left it and proceeds. The convention holds at every site with a single consistent shape; `tests/test_utils/server.rs`'s `buffer_text` shows it.

## Cancellation of a Job

A Job ends when its job entry goes. A stop, a reset and a delete of its Game remove the entry. Before each step, the Job writes its step name through its `JobWriter`, and that write checks the job id, so a Job with no entry starts no further step.

A stop can reach a Job while an LLM call runs, and the call returns after it. The same job id check covers that case: each save of the Job compares its id with the job entry of the Game, and a stopped Job has no entry. The check refuses the save, and the Job ends with no write. The Job code stays synchronous, because a late answer cannot change a record.

The server stops accepting requests when the process receives the shutdown signal, and it serves the requests it already holds until they end.

## Document References

- `../reference/architecture_system.md` — tier map (the canonical home for the settings resolution shape) + invariant identifiers.
- `../reference/coding_standards/guardrails.md` — the static guardrail layers (clippy, arch-lint, syn-based convention tests) that enforce the conventions above.
- `../reference/narrative/narration_system.md` — LLM connections and their token caps, including the `max_context_tokens` context window.
- `../reference/game_flow.md` — the Job, its job entry and the writer rule that the cancellation above relies on.
- `../explanation/architecture.md` [§Architectural commitments](../explanation/architecture.md#architectural-commitments) — the quality attributes the conventions above guarantee.
