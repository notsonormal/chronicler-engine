---
diataxis: explanation
title: Two State Channels
---

## The two signals

The engine carries two representations of generation state. One answers "is something generating right now?" on a hot path that runs many times per second. The other answers "what was the durable generation phase when this state was committed?" after a panic, restart, or debug inspection. The two signals serve different consumers and carry different cost profiles, so the engine keeps both, and recovery repairs the stale direction of their disagreement.

## Channel A — the persisted status

`state.narrative.input_buffer.status: GenerationStatus` lives in the game state and is persisted with every snapshot and message write. The persisted status survives panics, process restarts, and crashes. Recovery after an abnormal exit reads this field: if it still shows `Generating` while the runtime claims no work in flight, the next action concludes the previous run died mid-flight and resets the status to `Idle` before proceeding.

The persisted status is the channel debug operators read when a game appears stuck. It is the only signal that survives across processes.

## Channel B — the process-local registry

`GenerationGate` owns a per-game slot registry (`Arc<RwLock<HashMap<u64, GenerationSlot>>>`) that tracks which generations are live in this process. Handlers and the HTTP poll endpoint read it through `GenerationGate::is_busy` to reject double-spawn attempts without a storage round-trip. An RAII guard (`GenerationGuard::Drop`) releases the slot on a normal return and on panic alike, so a generation interrupted by a panic still frees the slot the next time it is observed.

The registry is process-local by design. The engine's deployment contract is one process against one database; a second engine process against the same database would hold its own registry and would not see the first process's claim through it.

## Where each channel is read

The two signals answer different questions for different code paths:

- **UI display** reads the registry. The status fragment, the options dock and `GET /debug/is_generating` all answer "is something generating?" with `GenerationGate::is_busy` for the current game, so a stale persisted `Generating` cannot lock the page; the persisted status still supplies the phase name and the failure the fragment renders.
- **Spawn-side concurrency gating** reads the registry. Handlers reject double-spawn attempts on an O(1) check that does not contend with the storage layer.
- **Self-healing recovery** compares the persisted status against the registry. The engine heals on boot, and on the next entry point after a panic: free action, options refresh, retrigger, retry, or game switch. When the persisted status says `Generating` and the registry holds no slot, the engine treats the previous run as a mid-flight crash. It resets the status to `Idle`.
- **Cross-process coordination** is supported by the persisted status only. The registry cannot coordinate across processes because each process holds its own; the deployment contract is one process per database.

## Who writes each channel

Each channel has several writers, and the two writes of one generation are sequential rather than atomic. `GenerationGate::try_claim` inserts the registry slot under the registry write lock, releases that lock, and only then writes the persisted `Generating`. `GenerationGuard::Drop` releases the slot and leaves the persisted status to the pipeline's finalize, cancellation, and error paths.

Arrival narration takes a second path. `ArrivalTaskContext` sets the persisted `Generating` in memory, but its only save runs after the status is already `Idle` or `Error`, so the persisted record never shows arrival as live. Arrival also takes no registry slot.

The two channels therefore disagree in two windows. A live slot can sit beside a persisted `Idle` while a generation finishes, because the pipeline writes `Idle` before its guard drops. A persisted `Generating` can sit beside no slot after a panic or a kill, because one path writes the status and another path releases the slot.

Agreement between the two channels is a convention, not a tested invariant. No test asserts that a live slot and a persisted `Generating` always coincide, because they do not. Recovery repairs the stale direction only: `GenerationGate::heal_stale` resets a persisted `Generating` that no slot owns, and the boot path applies the same heal before the server starts.

## Document References

- [`../reference/game_flow.md`](../reference/game_flow.md) — the factual phase sequence and the granular status-phase table.
