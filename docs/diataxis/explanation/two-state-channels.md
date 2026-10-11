---
diataxis: explanation
title: Two State Channels
---

## The two signals

The engine carries two representations of generation state. One answers "is something generating right now?" on a hot path that runs many times per second. The other answers "what was the durable generation phase when this state was committed?" after a panic, restart, or debug inspection. The two signals serve different consumers and carry different cost profiles, so the engine keeps both, and recovery repairs the stale direction of their disagreement.

## Channel A — the persisted status

`state.narrative.input_buffer.status` lives in the game state and is persisted with every snapshot and message write. The persisted status survives panics, process restarts, and crashes.

## Channel B — the process-local registry

`GenerationGate` owns a per-game slot registry that tracks which generations are live in this process. An RAII guard (`GenerationGuard::Drop`) releases the slot on a normal return and on panic alike.

The registry is process-local by design. The engine's deployment contract is one process against one database.

## Where each channel is read

The UI display reads the registry so that a stale persisted `Generating` cannot lock the page.

## Where the channels disagree

The two channels disagree in two windows. A live slot can sit beside a persisted `Idle` while a generation finishes, because the pipeline writes `Idle` before its guard drops. A persisted `Generating` can sit beside no slot after a panic or a kill, because one path writes the status and another path releases the slot.

## Document References

- [`../reference/game_flow.md`](../reference/game_flow.md) — the phase sequence and which channel each reader uses.
