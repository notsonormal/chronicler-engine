---
diataxis: explanation
title: Two State Channels
---

> **Diátaxis mode:** Explanation. This document is *understanding-oriented*: it explains why generation state is represented by two complementary signals rather than one. The factual shape of the channels — when each phase is active, what UI text appears, what the polling endpoint returns — lives in [`../reference/game_flow.md`](../reference/game_flow.md); this document is about the tradeoff those channels encode.

## The question

A text-adventure engine needs to answer two questions that look similar but operate on different timescales:

1. **Right now, is something generating?** — asked by the HTTP poll endpoint on every page refresh, often many times per second.
2. **What is the durable generation phase, and what was the last error?** — asked after a panic, a restart, or when a debug operator inspects a stuck game.

These questions have different consumers, different latency budgets, and different durability requirements. A single signal cannot answer both cheaply, and the Chronicler Engine therefore carries two.

## What each channel is

**Channel A — the persisted status field.** `state.narrative.input_buffer.status: GenerationStatus` is part of the game state and is persisted with every snapshot and message write. It survives panics, process restarts, and crashes. The persisted status is the source of truth for recovery and debugging: after any abnormal exit, the next action reads this field, and if it still shows `Generating` while the runtime claims otherwise, the engine knows the previous run died mid-flight and heals the state.

**Channel B — the process-local atomic flag.** `is_generating: Arc<AtomicBool>` lives on the application service and is a cached projection of the per-game registry (`Arc<RwLock<HashMap<GameId, GenerationSlot>>>`). It is read on the hot poll path with a single atomic load — no lock, no storage round-trip. It is cleared by an RAII guard (`GenerationGuard::Drop`) so a panic mid-generation still releases the flag, and it is consulted by handlers to reject double-spawn attempts before they reach the pipeline.

## Why two and not one

The temptation is to collapse to a single source: the persisted status, queried on every poll. That would work, but at a cost the engine has explicitly decided not to pay.

The poll endpoint is the hottest read path in the application. Reading from SQLite on every poll — a query that runs dozens of times per second per active browser — would dominate request latency and burn through the connection-pool budget for no semantic benefit. The persisted status is good for *recovery* (one query when a process starts) and *debugging* (one query when a human looks) but bad for *steady-state polling* (hundreds of queries per second per session).

So the atomic flag exists as a read-mostly cache, kept consistent with the persisted status under a documented single-writer rule: only the registry claim/release path mutates both representations, and it does so under the same write-lock scope so no observer sees them disagree. The `GenerationGuard::Drop` is a second writer for the atomic's `false` transition only — the RAII panic-safety fallback. The persisted status's `true` transitions are owned solely by the application service's action path.

A property test (encoded as an invariant contract test) verifies the invariant: after any mutation, `AtomicBool.load() == (persisted_status == Generating)`. Drift between the two representations fails the test suite rather than shipping.

## How they relate without duplicating

The two channels answer different questions and are consumed by different code paths:

- **UI display** — driven by the persisted status (via the polled status fragment), because the UI must show the correct state even after the page was loaded from a stale tab or a crashed browser.
- **Spawn-side concurrency gating** — driven by the atomic flag, because handlers need an O(1) check that does not contend with the storage layer.
- **Self-healing recovery** — driven by the persisted status. On the next `process_action` after a panic, the engine compares persisted status against the atomic. If they disagree (persisted says `Generating`, atomic says `false`), it is evidence the previous run crashed mid-flight; the engine resets status to `Idle` and proceeds.
- **Cross-process coordination** — impossible by design. The atomic flag is process-local. Two engine processes pointed at the same database would each maintain their own atomic, and neither would see the other's claim. The persisted status is the only signal that survives across processes; the engine does not run multiple processes against one database in production.

## What this forbids

Because two representations of one fact are a documented divergence risk, the architecture contract is explicit about what other code paths may do:

- Any code path other than the registry claim/release path that mutates the `true` atomic transition, or the persisted `GenerationStatus` `true` transition, is a bug.
- Any code path other than `GenerationGuard::Drop` that mutates the `false` atomic transition is a bug.
- Reading the atomic is permitted everywhere; writing it is not.

The property test is the binding safety mechanism. If it is ever deleted or weakened, the invariant degrades from "machine-checked" to "convention only".

## What this design does not address

The dual-channel design assumes a single process. There is no cross-process coordination on the atomic flag. Multi-process deployments against a shared database would need a different gate (a database-level lock, or a distributed cache), and that design has not been considered. The single-process assumption is enforced upstream by the deployment story (one engine binary per SQLite file).

## See also

- [`../reference/game_flow.md`](../reference/game_flow.md) — the factual phase sequence and the granular status-phase table.
- [ADR-010: Concurrency and Generation Gate Model](../../docs/adr/adr-010-concurrency-generation-gate.md) — the original generation-gate decision (tokio migration + atomic + RAII guard).
- [ADR-030: `is_generating` Dual-Source Invariant](../../docs/adr/adr-030-is-generating-invariant.md) — the formal dual-source contract, single-writer rule, and verification strategy.
