# Decide the storage seam: the in-memory adapter, the story projections and the read failures

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The `InMemory` backend can no longer create a Game, and 14 arms only refuse. What is the storage seam? Decide:

1. **Delete the in-memory adapter** (SQLite `:memory:` is the one stand-in) **or split the seam** along the catalogue and story line (candidate M2, finding 7). Either way, the unit-tier row of `tests/STRATEGY.md` follows.
2. **The read-failure injection**: string-keyed today, against the typed `TxWrite` for writes; and the storage seam's missing read-failure case.
3. **One story entry instead of three projections** (`StoryRow`, `MessageEntry`, `StoryEntry`; candidate M5), where serde and `agent-edge` allow it.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) rows M2, M5; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) finding 7 and the storage-seam low finding; the three runs differ here (Opus deletes, GLM and Qwen split).
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/turn-state-model/assets/target-design.md`.

## Answer
