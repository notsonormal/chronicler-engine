---
diataxis: explanation
title: Rust Idioms
---

## Concrete services

`Job`, `GameWriter`, the catalogue collaborators and `Storage` are concrete structs. Trait objects sit only where the polymorphism is the point, such as the Agents that `AgentRunner` dispatches.

The application code is synchronous. Async I/O stays in the HTTP layer, and the application runs on the blocking pool (next section). So the storage and service types need no async trait, no type parameter threaded through the application, and no per-test mock.

## LLM-call offload via spawn_blocking

`Job::start` accepts the request and spawns the Job on the blocking pool. The handler gets its answer when the accept succeeds, and the Job runs on after the response.

The offload separates the Axum event loop from the LLM network call, which can take seconds. One slow LLM call does not back up unrelated handlers, and the synchronous service code runs unchanged inside the blocking pool.

## Settings resolution at the read site

Components read settings at the point of use and pass the value down the call. Nothing holds a long-lived settings handle. The shape serves a read-mostly workload: each read is a short storage call, and writes are rare dashboard requests. `Storage::update_settings` does the read, the change and the write under one storage acquisition, so two overlapping edits cannot overwrite each other.

## Lock-poison recovery

Every `Mutex` site recovers from a poisoned lock with `.into_inner()`. Poisoning is the standard library's signal that a previous holder panicked. That panic is a bug, but the engine treats the next holder as legitimate: it gets the guarded value as the previous holder left it. One panicking request therefore cannot take the lock down for the life of the process.

## Cancellation of a Job

A stopped Job is not interrupted. It finds out at its next step or its next save, because both check its job id (see the Game Flow reference). A late LLM answer therefore cannot change a record, and the Job code stays synchronous with no cancellation points.

The server stops accepting requests when the process receives the shutdown signal, and it serves the requests it already holds until they end.

## Document References

- [`./architecture.md`](./architecture.md) — the dependency rule and the deployment contract that these idioms sit inside.
- [`../reference/game_flow.md`](../reference/game_flow.md) — the Job, its job entry and the id check that cancellation relies on.
