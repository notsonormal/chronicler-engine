---
diataxis: explanation
title: Two State Channels
---

## The two signals

The engine carries two representations of generation state. One answers "is something generating right now?" on a hot path that runs many times per second. The other answers "what was the durable generation phase when this state was committed?" after a panic, restart, or debug inspection. The two signals serve different consumers and carry different cost profiles, so the engine keeps both and maintains their consistency with an explicit invariant.

## Channel A — the persisted status

`state.narrative.input_buffer.status: GenerationStatus` lives in the game state and is persisted with every snapshot and message write. The persisted status survives panics, process restarts, and crashes. Recovery after an abnormal exit reads this field: if it still shows `Generating` while the runtime claims no work in flight, the next action concludes the previous run died mid-flight and resets the status to `Idle` before proceeding.

The persisted status is the channel debug operators read when a game appears stuck. It is the only signal that survives across processes.

## Channel B — the process-local atomic

`is_generating: Arc<AtomicBool>` lives on the application service and is a cached projection of the per-game registry. Handlers and the HTTP poll endpoint read it on every request with a single atomic load — no lock acquired, no storage round-trip. An RAII guard (`GenerationGuard::Drop`) clears the flag on a normal return and on panic alike, so a generation interrupted by a panic still releases the flag the next time it is observed.

The atomic is process-local by design. The engine's deployment contract is one process against one database; a second engine process against the same database would hold its own atomic and would not see the first process's claim through it.

## Where each channel is read

The two signals answer different questions for different code paths:

- **UI display** reads the persisted status (via the polled status fragment). The UI shows the correct state even when a tab was loaded before a crash or when the page renders from a stale snapshot.
- **Spawn-side concurrency gating** reads the atomic. Handlers reject double-spawn attempts on an O(1) check that does not contend with the storage layer.
- **Self-healing recovery** reads the persisted status. On the next `process_action` after a panic, the engine compares persisted status against the atomic; disagreement (persisted says `Generating`, atomic says `false`) is evidence of a mid-flight crash, and the engine resets the persisted status to `Idle`.
- **Cross-process coordination** is supported by the persisted status only. The atomic cannot coordinate across processes because each process holds its own; the deployment contract is one process per database.

## The single-writer rule

The two signals are kept coherent by restricting who may write each one. Only the registry claim/release path mutates both representations, and it does so under the same write-lock scope so no observer sees them disagree. `GenerationGuard::Drop` is a second writer for the atomic's `false` transition only — the RAII panic-safety fallback. The persisted status's `true` transitions are owned solely by the application service's action path. All other code paths treat both signals as read-only.

A property test (an invariant contract test) verifies the rule: after any mutation, `AtomicBool.load() == (persisted_status == Generating)`. Drift between the two representations fails the test suite rather than shipping. If the property test is ever deleted or weakened, the invariant degrades from machine-checked to convention only.

## Document References

- [`../reference/game_flow.md`](../reference/game_flow.md) — the factual phase sequence and the granular status-phase table.
- [ADR-010: Concurrency and Generation Gate Model](../../docs/adr/adr-010-concurrency-generation-gate.md) — the original generation-gate decision (tokio migration + atomic + RAII guard).
- [ADR-030: `is_generating` Dual-Source Invariant](../../docs/adr/adr-030-is-generating-invariant.md) — the formal dual-source contract, single-writer rule, and verification strategy.
