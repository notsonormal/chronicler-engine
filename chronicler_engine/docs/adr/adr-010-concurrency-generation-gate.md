# ADR-010: Concurrency and Generation Gate Model

**Date:** 2026-05-10
**Status:** Accepted

---

## Context

The engine used `std::thread::spawn` for all LLM work, creating OS threads invisible to the Tokio runtime. These threads could not be cancelled and silently poisoned mutexes on panic. When Phase 1.7 replaced `Arc<Mutex<GameState>>` with SQLite snapshots, the application-level serialization was lost, exposing a read-modify-write race in `process_action`.

Additionally, `std::thread::sleep(Duration::from_millis(50))` hacks were used to let "inner threads" start before HTTP responses returned.

---

## Decision

**Remove all `std::thread::spawn` from production code and replace with `tokio::task::spawn_blocking`, plus an `AtomicBool` generation gate for domain-level action serialization.**

### Tokio Migration

All blocking work moves into the async runtime's blocking pool. The LLM backend interface stays synchronous; the HTTP layer is responsible for non-blocking execution. Thread sleep hacks are removed.

### Generation Gate

An atomic flag in application state acts as a domain-level action lock:

- Set before accepting any player action
- Cleared automatically when the generation task finishes, even on panic (RAII guard)
- Client-side: HTMX sync attributes + button disable prevent most double-submits

---

## Consequences

### Positive
- **Runtime observability**: `spawn_blocking` tasks are visible to Tokio
- **No mutex poisoning**: `AtomicBool` + RAII guard is panic-safe
- **Domain semantics**: "Can't act while generating" is correct for a text adventure
- **Simpler tests**: Mock backends are synchronous; no thread timing issues

### Negative
- **Runtime dependency**: Requires active Tokio runtime (guaranteed by Axum)
- **Cooperative cancellation only**: Blocking tasks cannot be forcibly killed; they must poll a cancellation token at internal checkpoints to abort early. The pipeline checks at stage boundaries (post-narration, pre-trigger, post-trigger) and resets status to idle before returning.

### Trade-offs
- Chose `spawn_blocking` over `tokio::spawn` + `async` traits because backend traits are sync and `dyn async Trait` is complex in Rust 2024
- Chose generation gate over mutex because it rejects rather than queues, matching single-player semantics

---

## Related ADRs

- [ADR-008: SQLite Snapshot Persistence](./adr-008-sqlite-snapshot-persistence.md) — Snapshots removed the old mutex; generation gate replaced it

---

## History

- **2026-05-05**: Initial plan to replace `std::thread::spawn` with Tokio
- **2026-05-10**: Generation gate added to fix race condition exposed by snapshot migration
- **2026-05-18**: Cooperative cancellation checkpoints added to the action pipeline
