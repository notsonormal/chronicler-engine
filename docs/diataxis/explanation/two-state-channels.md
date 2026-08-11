---
diataxis: explanation
title: Two State Channels
---

## The two signals

The engine carries two representations of generation state. One answers "is something generating right now?" on a hot path that runs many times per second. The other answers "what was the durable generation phase when this state was committed?" after a panic, restart, or debug inspection. The two signals serve different consumers and carry different cost profiles, so the engine keeps both and maintains their consistency with an explicit invariant.

## Channel A — the persisted status

`state.narrative.input_buffer.status: GenerationStatus` lives in the game state and is persisted with every snapshot and message write. The persisted status survives panics, process restarts, and crashes. Recovery after an abnormal exit reads this field: if it still shows `Generating` while the runtime claims no work in flight, the next action concludes the previous run died mid-flight and resets the status to `Idle` before proceeding.

The persisted status is the channel debug operators read when a game appears stuck. It is the only signal that survives across processes.

## Channel B — the process-local registry

`GenerationGate` owns a per-game slot registry (`Arc<RwLock<HashMap<u64, GenerationSlot>>>`) that tracks which generations are live in this process. Handlers and the HTTP poll endpoint read it through `GenerationGate::is_busy` to reject double-spawn attempts without a storage round-trip. An RAII guard (`GenerationGuard::Drop`) releases the slot on a normal return and on panic alike, so a generation interrupted by a panic still frees the slot the next time it is observed.

The registry is process-local by design. The engine's deployment contract is one process against one database; a second engine process against the same database would hold its own registry and would not see the first process's claim through it.

## Where each channel is read

The two signals answer different questions for different code paths:

- **UI display** reads the persisted status (via the polled status fragment). The UI shows the correct state even when a tab was loaded before a crash or when the page renders from a stale snapshot.
- **Spawn-side concurrency gating** reads the atomic. Handlers reject double-spawn attempts on an O(1) check that does not contend with the storage layer.
- **Self-healing recovery** reads the persisted status. On the next `process_action` after a panic, the engine compares persisted status against the atomic; disagreement (persisted says `Generating`, atomic says `false`) is evidence of a mid-flight crash, and the engine resets the persisted status to `Idle`.
- **Cross-process coordination** is supported by the persisted status only. The atomic cannot coordinate across processes because each process holds its own; the deployment contract is one process per database.

## The single-writer rule

The two signals are kept coherent by restricting who may write each one. Only the registry claim/release path mutates both representations, and it does so under the same write-lock scope so no observer sees them disagree. `GenerationGuard::Drop` is a second writer for the registry slot release only — the RAII panic-safety fallback. The persisted status's `true` transitions are owned solely by the action path in `ActionPipeline`. All other code paths treat both signals as read-only.

An invariant contract test verifies the rule: after any mutation, the registry state agrees with the persisted status (`GenerationGate` has a generating slot for the current game iff `state.narrative.input_buffer.status == Generating`). Drift between the two representations fails the test suite rather than shipping. If the test is ever deleted or weakened, the invariant degrades from machine-checked to convention only.

## Document References

- [`../reference/game_flow.md`](../reference/game_flow.md) — the factual phase sequence and the granular status-phase table.
